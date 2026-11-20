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
    // S1-C-12 wave D — the completed ammo sets' cost flags
    // (`ArmorSetBonuses.Benefits.Fossil`/`CobaltRanged`/`MythrilRanged` →
    // `ammoCost80`, `AdamantiteRanged` → `ammoCost75`,
    // `ArmorSetBonuses.cs:337-340`/`:174-177`/`:189-192`/`:205-208`). The
    // C# grants them inside `UpdateArmorSets` (`Player.cs:25685`), but the
    // port's clear above runs AFTER that update in the tick — so the grant
    // lives here, after the clear, before the shoot path reads the flags.
    crate::armor_sets::grant_set_ammo_flags(p, armor);
}

/// The mechanical-accessory half of `Player.UpdateEquips` (S1-P-i34) — the
/// wire-visibility goggles and the auto-actuator. The flags clear in
/// `ResetEffects` (`Player.cs:18524`/`:18534`, run inside `Player::update`)
/// and re-derive here from this tick's equipment, in the C#'s scan order.
///
/// `RefreshMechanicalAccsFromItemType` (`Player.cs:12799-12820`) is called
/// from three places: the 58-slot inventory scan (`UpdateEquips`,
/// `:12900-12914`), `GrantArmorBenefits` over the ten functional armor slots
/// (`:12919-12930` → `:13103-13107`), and `ApplyEquipFunctional` over the
/// accessory slots (`:12946-12951`). The port folds the inventory + armor
/// scopes into one pass (the flags are idempotent ORs, so order is immaterial);
/// the held-item arm (`ApplyEquipFunctional`'s `currentItem.type == 3624`,
/// `:15295-15298`) runs on the selected slot.
///
/// Scope: the Mechanical Lens (3619) and Wire Kite (3611) grant
/// `InfoAccMechShowWires` (`:12801-12804`); the Actuation Accessory (3624)
/// grants `autoActuator` (`:12817-12820`). The Wire Kite's ruler-line/grid
/// halves (`:12805-12812`) and the auto-paint family are out of this wiring
/// wave. The `autoActuator` placement consumer
/// (`PlaceThing_Tiles_PlaceIt_AutoPaintAndActuate`, `:40594`) is a recorded
/// follow-up — the flag is granted and read by the visibility gate, but the
/// auto-place-on-build pass is not ported (ANALYSIS "The wiring tools").
pub(crate) fn grant_mechanical_accs(
    p: &mut Player,
    inventory: &[ItemSlot],
    armor: &[ItemSlot; ARMOR],
    held_kind: u16,
) {
    // The inventory scan (`UpdateEquips`, `Player.cs:12900-12914`): the port's
    // 50-slot grid plus the coin/ammo rows the C# keeps in the same 58-slot
    // array. The caller hands the grid; the coin/ammo rows carry no mech
    // accessory, so the grid scan is the whole reachable set.
    for slot in inventory {
        refresh_mechanical_acc_from_item_type(p, slot.kind);
    }
    // `GrantArmorBenefits` over the ten functional armor slots
    // (`GetEffectiveArmor(0..10)`, `Player.cs:12919-12930`): 3 armor + 7
    // accessories. The vanity half (slots 10..20) grants nothing here.
    for slot in &armor[..10] {
        refresh_mechanical_acc_from_item_type(p, slot.kind);
    }
    // The held-item arm (`ApplyEquipFunctional`, `Player.cs:15295-15298`):
    // the Actuation Accessory in hand also grants `autoActuator`.
    refresh_mechanical_acc_from_item_type(p, held_kind);
}

/// `Player.RefreshMechanicalAccsFromItemType` (`Player.cs:12799-12820`),
/// the wiring-relevant arms only.
fn refresh_mechanical_acc_from_item_type(p: &mut Player, kind: u16) {
    // `:12801-12804` — the Mechanical Lens / Wire Kite show the wires.
    if kind == 3619 || kind == 3611 {
        p.info_acc_mech_show_wires = true;
    }
    // `:12817-12820` — the Actuation Accessory auto-places actuators.
    if kind == 3624 {
        p.auto_actuator = true;
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
        // S1-P-i27 — the whip-range armor halves beside the cap rows:
        // `:13852-13855` the Tiki Mask (1159) `whipRangeMultiplier +=
        // 0.1f`; `:13876-13881` the Stardust Plate/Leggings (3382/3383)
        // `+= 0.15f`. (The rows' `minionDamage` halves ride the
        // `set_stats`/combat folds; the cap halves above.)
        match slot.kind {
            1159 => p.whip_range_multiplier += 0.1,
            3382 | 3383 => p.whip_range_multiplier += 0.15,
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
    // The `useTurn` direction lock's inputs (`Player.cs:19837`): the swing
    // state and the held item's `useTurn` flag, both read fresh each tick.
    let item_in_use = app.swing > 0.0;
    let held_use_turn = crate::registry::get(app.inventory[app.selected].kind)
        .is_some_and(|d| d.use_turn);
    if map_open {
        return PlayerInput {
            left: false,
            right: false,
            jump: false,
            down: false,
            up: false,
            hook: false,
            item_in_use,
            held_use_turn,
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
        // `Player.controlUp` on `Main.cUp` (`Main.cs:67163`, default `W`).
        up: any(crate::input_map::Action::Up),
        // `Player.controlHook` on `Main.cHook` (`Main.cs:67172`, default `E`);
        // the port keeps `G` as the extra quick-grapple key.
        hook: any(crate::input_map::Action::Hook) || app.keys.contains(&KeyCode::KeyG),
        item_in_use,
        held_use_turn,
    }
}

/// S1-P-i55 — `Main.LocalGolfState.Update()` (`Main.cs:17219`,
/// `GolfState.cs:111-133`): once per fixed world tick, ahead of the player
/// body (the C# call precedes `UpdatePlayers`). The held item's type/shoot
/// gate the settle-wait (`IsAGolfingItem` + the 3611 whistle); the live
/// ball is the pool slot the state's stored snapshot names, re-validated
/// by `GetLastHitBall`'s four guards (`GolfState.cs:102-109`). The parts
/// signature keeps the stepper closures' `&mut Player` borrow disjoint.
fn update_golf_state(
    golf_state: &mut tr_core::golf_state::GolfState,
    inventory: &[crate::item::ItemSlot],
    selected: usize,
    projectiles: &crate::projectile::ProjectilePool,
    local_id: i32,
) {
    let held = inventory
        .get(selected)
        .filter(|s| s.count > 0)
        .map(|s| s.kind)
        .unwrap_or(0);
    let held_shoot = crate::registry::projectile(held).unwrap_or(0) as i32;
    let live_ball = golf_state.last_hit_golf_ball().and_then(|stored| {
        projectiles.slots().get(stored.who_am_i).map(|q| {
            let (cx, cy) = q.center();
            tr_core::golf_state::GolfBall {
                who_am_i: stored.who_am_i,
                kind: q.kind as i32,
                owner: q.owner as i32,
                active: q.active,
                position: q.pos,
                center: Vec2::new(cx, cy),
                ai1: q.ai[1],
                local_ai1: q.local_ai[1],
            }
        })
    });
    golf_state.update(held as i32, held_shoot, local_id, live_ball.as_ref());
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
    // `Player.cs:24834-24845`: the `controlQuickMana` press edge fires
    // `QuickMana` once (the `releaseQuickMana` latch); the port keeps the
    // edge on the App like `mine_prev_down`. The QuickHeal twin
    // (`:24822-24833`) is the i54 hotbar worker's seam — not duplicated.
    // The C# zeroes the movement controls while the map is fullscreen
    // (`:24772-24808`) but NOT the quick-use controls, so the key stays
    // live under the open map.
    let mana_key = app
        .keys
        .iter()
        .any(|k| crate::input_map::is_bound(&binds, crate::input_map::Action::QuickMana, *k));
    let mana_edge = mana_key && !app.quick_mana_prev;
    app.quick_mana_prev = mana_key;
    if mana_edge {
        crate::potions::quick_mana(app);
    }
    let (sw, sh) = app.screen_size();
    let mouse_world = Vec2::new(
        app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
        app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
    );
    // S1-P-i27 — `Main.ReversedUpDownArmorSetBonuses` (`Main.cs:273`), copied
    // out before the player borrow (the rest-target command reads it).
    let reversed_armor_bonuses = app.settings.reversed_armor_bonuses;
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
    // The dust spawn ctx (S1-E-1 phase 2), copied out before the player
    // borrow so the closure stays on disjoint field captures; the front
    // state and the pause flag cannot change inside the player tick.
    let spawn_ctx = app.spawn_ctx();
    // The `59 HitSwitch` sends the tick's switch scans queue
    // (`Collision.cs:3304`/`:3318`/`:3324`/`:3347`): collected here because
    // the C# fires them from inside `Player.Update` but the port's closure
    // holds `app` — they go out once it releases the borrow.
    let mut switch_sends: Vec<(i32, i32)> = Vec::new();
    // S1-P-i54 — `TriggersSet.CopyInto` (`TriggersSet.cs:844-847`): the
    // left button up clears `Main.blockMouse` every tick (the port's
    // `mine_down` is the button state the winit arm keeps current).
    if !app.mine_down {
        app.block_mouse = false;
    }
    // ---- S1-P-i54: the quick-heal key (`Main.cHeal`, default H) ----
    // `Player.cs:24822-24833`: the `controlQuickHeal` press edge fires
    // `QuickHeal` once (the `releaseQuickHeal` latch); the port keeps the
    // edge on the App like `mount_prev`. The C# order puts QuickHeal
    // (`:24826`) before QuickMount (`:24890`) and QuickGrapple (`:27486`),
    // so the drink runs before the main player borrow below. The player
    // is taken out for the call (the fn reads the banks and writes the
    // App's audio/combat-text/rng halves).
    let heal_key = app
        .keys
        .iter()
        .any(|k| crate::input_map::is_bound(&binds, crate::input_map::Action::QuickHeal, *k));
    let heal_edge = heal_key && !app.quick_heal_prev;
    app.quick_heal_prev = heal_key;
    if heal_edge && !map_open {
        // The canonical S1-P-i37 drink (`potions::quick_heal`) — it reads
        // the player through `app.player` (the banks, the gates) and owns
        // the whole `:5501-5547` chain incl. the cursed gate + the pStone
        // delay scaling. Keep-one reconciliation (gate V4): the i54 inline
        // copy is retired.
        crate::potions::quick_heal(app);
    }
    // ---- S1-P-i54: the selection machine (`Player.selectedItemState`) ----
    // `Player.Update`'s `selectedItemState.Update()` (`Player.cs:25218`)
    // applies the buffered switch only when idle, then resets the use
    // timers (`OnSelectionChanged`, `:551-567`). The digit/wheel/click
    // handlers buffer the request (`Select`, `:486-518`); this is the
    // apply half. `CanChangeSelectedItemImmediately` (`:428-438`) =
    // `!UsingOrReusingItem && ItemTimeIsZero`; `UsingOrReusingItem`
    // (`:4068-4078`) = `itemAnimation > 0 || reuseDelay > 0 || channel ||
    // pendingItemReuse` (the last has no port field).
    {
        // `CanChangeSelectedItemImmediately` reads the channel state as
        // it stands at `selectedItemState.Update` time — the C# runs
        // Update (`Player.cs:25218`) BEFORE ItemCheck (`:28285`), so the
        // interrupt below (ItemCheck's arm) cannot unblock the switch on
        // the same tick; the switch lands the NEXT tick.
        let channel = app.player.as_ref().is_some_and(|p| p.channel);
        let can_change =
            app.swing <= 0.0 && app.reuse_delay <= 0.0 && !channel && app.item_time <= 0.0;
        // `ItemCheck` (`Player.cs:42800-42807`): a buffered change forces
        // `flag5 = false`, and `!flag5` clears `channel` the SAME tick —
        // the switch itself then lands next tick once `channel` is off.
        // The interrupt clears MAIN's channel seam (`Player.channel` /
        // `channel_item` / `channel_target`, `player.rs:1395-1411`, the
        // aiStyle-9 owner-block hold reads them) — NOT the i20 branch's
        // future `channel_shoot` field (Lead decision, ANALYSIS
        // "Hotbar/quick-switch (S1-P-i54)"); clearing the state works
        // both pre- and post-i20.
        if app.selection.has_buffered_change() {
            if let Some(p) = app.player.as_mut() {
                if p.channel {
                    p.channel = false;
                    p.channel_item = 0;
                    p.channel_target = Vec2::ZERO;
                }
            }
        }
        // `PickItemSelectionOverride` (`Player.cs:17410-17432`): the
        // mouse-item arm (`:17412-17416`, slot 58) has no port (the live
        // slot indexes the 50-grid); the smart-select arm is the override.
        // `controlTorch` = the LeftShift auto-select key (`Main.cTorch`).
        let control_torch = app
            .keys
            .iter()
            .any(|k| crate::input_map::is_bound(&binds, crate::input_map::Action::Torch, *k));
        let override_slot = if can_change {
            app.player.as_ref().and_then(|p| {
                let (sw, sh) = app.screen_size();
                let mouse = (
                    app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
                    app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
                );
                let ctx = crate::hotbar::SmartSelectCtx {
                    world: &app.world,
                    inv: &app.inventory,
                    coins: &app.equip.coins,
                    ammo: &app.equip.ammo,
                    mouse,
                    player_pos: (p.x, p.y),
                    player_h: p.height,
                    wet: p.wet,
                };
                crate::hotbar::smart_select_lookup(&ctx, control_torch, app.selected)
            })
        } else {
            None
        };
        if let Some((new_sel, continue_auto_reuse)) =
            app.selection
                .update(app.selected, can_change, &app.inventory, override_slot)
        {
            app.selected = new_sel;
            // `OnSelectionChanged` (`Player.cs:551-567`):
            // `itemTime = itemTimeMax = 0` (`:553`). The switch only
            // applies when idle, so the swing is already 0 — the reset
            // clears any stale action accumulator the same way.
            // `lastItemUseAttemptSuccess = true` (`:554`) has no port
            // field (the net-sync latch, `:6902`).
            app.item_time = 0.0;
            if app.mine_down {
                if continue_auto_reuse {
                    // `:557-560`: `releaseUseItem = true` — the non-
                    // autoReuse gate reads the prev-edge pair, so arm both
                    // halves of the "was released" state for this tick.
                    app.mine_released = true;
                    app.mine_prev_down = false;
                } else if !app.mine_released {
                    // `:561-565`: `Main.blockMouse = true; controlUseItem
                    // = false`. The port folds controlUseItem into
                    // mine_down (no re-press event arrives while held, so
                    // it stays false until the release — the C# re-arms
                    // only when blockMouse clears on release,
                    // `TriggersSet.cs:844-847`).
                    app.block_mouse = true;
                    app.mine_down = false;
                }
            }
        }
    }

    // `Player.Shellphone_Spawn`'s invalid-area force-clear request
    // (`Player.cs:56169-56172`): collected in the tick, run after the player
    // borrow releases `app` (the kills + drop rolls need `&mut App`).
    let mut force_clear: Option<(i32, i32)> = None;
    if let Some(p) = app.player.as_mut() {
        // `Player.cs:18802-18804`: `ResetEffects` clears the Actuation Rod's
        // lock ONLY while `releaseUseItem` rides. `releaseUseItem` is
        // `!controlUseItem` latched at the tail of the previous ItemCheck
        // (`:42829`); the port's `mine_released` holds that state (refreshed
        // every tick by `update_editing`). A held button keeps the lock across
        // ticks — one actuation per hold, not per tick (the v1 cleared it
        // unconditionally, which let a held rod re-flip every tick).
        if app.mine_released {
            p.actuation_rod_lock = false;
        }
        // `Player.ResetEffects`' journey block, re-run every tick: the godmode
        // flag (`:18789` clears, `:25493-25497` sets) and the far-placement
        // tile-range toggle (`:18815-18828`).
        p.creative_god_mode = power_god_mode || p.debug_god_mode;
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
        // The buff bar's removal requests (`Main.TryRemovingBuff`,
        // `Main.cs:43348`'s draw-pass mutation — the draw pushed the slots,
        // the tick applies them): non-debuffs only, the mount's own buff
        // dismounts through `TryDismount`'s gates, the pet/light hide arms
        // read `miscEquips[0..2].buffType`, and `SoundEngine.PlaySound(12)`
        // (`:43382`) is the MenuTick cue. The C# draw runs alive or dead
        // (`DrawInterface_Resources_Buffs` has no dead gate), so the drain
        // sits BEFORE the dead branch — it used to live inside it, which
        // made every living player's cancel requests pile up unapplied.
        {
            let misc_buffs = [
                app.equip
                    .misc_equips
                    .first()
                    .filter(|s| s.count > 0)
                    .and_then(|s| crate::registry::get(s.kind))
                    .map_or(0, |d| d.buff_type),
                app.equip
                    .misc_equips
                    .get(1)
                    .filter(|s| s.count > 0)
                    .and_then(|s| crate::registry::get(s.kind))
                    .map_or(0, |d| d.buff_type),
            ];
            for slot in std::mem::take(&mut app.buff_removals) {
                let bt = p.buffs.buff_type[slot];
                if p.try_removing_buff(&app.world, slot, bt, misc_buffs) {
                    app.audio.play("Menu_Tick");
                }
            }
        }
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
            // `Main.snowMoon || Main.pumpkinMoon || DD2Event.Ongoing`
            // (`Player.cs:38984-38986`).
            let moon_event_live =
                app.world.snow_moon || app.world.pumpkin_moon || app.world.dd2_event_ongoing;
            app.respawn_skip_available =
                p.allows_respawn_timer_skip(boss_near, danger_near, moon_event_live);
            let mut respawn_now = false;
            app.player_step.tick(dt, |_| {
                // S1-P-i56 — `Player.Update`'s tank-pet decay
                // (`Player.cs:24539-24548`): the C# block sits AHEAD of the
                // dead-player early return (`:24749-24754`), so the dead
                // player's claim decays too — the Guardian's own dead-owner
                // arm clears the summon flag, the 2-tick despawn follows
                // and the claim drops one grace tick later.
                p.tank_pet_decay();
                // S1-P-i55 — the golf state update runs on the dead branch
                // too (`Main.cs:17219` is unconditional).
                update_golf_state(
                    &mut app.golf_state,
                    &app.inventory,
                    app.selected,
                    &app.projectiles,
                    app.local_id as i32,
                );
                respawn_now = p.update_dead(input.jump, boss_near, danger_near, moon_event_live)
                    || respawn_now;
                p.reset_projectile_caches();
                p.update_projectile_caches(&app.projectiles);
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
        // The edge is the C#'s per-player `releaseHook` latch
        // (`Player.cs:27481-27489`) — the port used a process-global
        // stand-in, which made the edge fire for the wrong player.
        let hook_edge = if input.hook {
            let edge = p.release_hook;
            p.release_hook = false;
            edge
        } else {
            p.release_hook = true;
            false
        };
        // `:6298-6301` — `CCed || tongued || dead` refuses the fire BEFORE any
        // dismount (`QuickGrapple` returns at the head). The mount dismount
        // (`:6334-6337`) sits INSIDE this gate — a CCed/tongued/dead player
        // keeps the mount (the C# never reaches `TryDismount`).
        if hook_edge && !p.dead && !p.cced() && !p.tongued {
            // `Player.QuickGrapple` (`Player.cs:6334-6337`): a mount not in the
            // CanUseHooks set dismounts before the hook fires — the set is
            // {54, 57-60} (`mount::can_use_hooks`).
            if p.mount.is_active() && !crate::mount::can_use_hooks(p.mount.kind()) {
                p.try_dismount(&app.world);
            }
            // `QuickGrapple` (`Player.cs:6296-6478`) in the source's order:
            // the item pick (`:6320`), the fire-time gate (`:6399`), the
            // blacklist rebuild (`:6407`), the UseSound (`:6408`), the
            // oldest-hook recycle (`:6416-6419`, `num7` of the ORIGINAL shoot),
            // THEN the Dual (`:6420-6428`) / Lunar (`:6430-6458`) kind cycles,
            // and only then the spawn (`:6460-6478`).
            let owner = app.local_id as u8;
            let (flight, attached) = app.projectiles.hook_counts(owner);
            // `Player.QuickGrapple_GetItemToUse` (`Player.cs:6481-6500`): the
            // grapple slot (the misc-equip 4) first, then the inventory — the
            // first item whose `shoot` is a grappling hook. The C# returns
            // when no hook item is held (`:6320-6323`); the port keeps the
            // `TR_HOOK` pin as the stand-in for the granting items (M3-EQUIP-3),
            // so the fire still happens without one.
            let item_hook = crate::grapple::quick_grapple_item_kind(
                &app.equip.misc_equips,
                &app.inventory,
            );
            let mut kind = item_hook.map(|(_, shoot)| shoot).unwrap_or(hook_type as u16);
            // `:6342-6398` — the budgets read the ORIGINAL `item.shoot` (`num`),
            // before the kind cycles rewrite it.
            let original_shoot = kind;
            // `:6399-6402` — the in-flight gate (`num4 >= max(1, num6 - num5)`).
            // The C# `return` exits `QuickGrapple` ONLY — the rest of
            // `Player.Update` (movement, the mount key, the pull bridge) still
            // runs. The port wraps the fire tail in the gate's inverse instead
            // of returning out of the whole player tick.
            if crate::grapple::fire_allowed(flight, attached, original_shoot) {
            // `:6407` — rebuild the owner's refused-tile set before the fire.
            app.projectiles.rebuild_hook_blacklist(&app.world, owner);
            // S1-E-21f — `QuickGrapple`'s fire sound (`Player.cs:6408`):
            // `PlaySound(item.UseSound, base.position)` — the hook item's
            // own use-sound row at the TOP-LEFT, after the fire-time gates
            // passed (the `:6399` budget return precedes it) and BEFORE the
            // recycle/spawn. The style façade's `GetRandomPitch` NextFloat
            // rides the audio-only stream; the case-2 pitch rides
            // `UpdatePlayers` (the fire runs inside `Player.Update`,
            // `Main.cs:17754`). No granting item (the `TR_HOOK` pin) means
            // no `UseSound` — silent, like the C#'s null-item return
            // (`:6320-6323`). KEPT from wave 2 (S1-E-21f).
            if let Some((item_kind, _)) = item_hook {
                if let Some((idx, pitch)) = crate::item::initial_use_sound(item_kind) {
                    app.audio.play_use_sound(
                        idx,
                        p.x,
                        p.y,
                        pitch,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                }
            }
            // `:6413-6415` — the spawn reads `item.shootSpeed`/`damage`/
            // `knockBack` (the per-item rows; the `TR_HOOK` pin keeps the
            // Hook's 11.5 / 0 / 7 stand-in until the granting items land).
            let (shoot_speed, damage, knock_back) = item_hook
                .and_then(|(item_kind, _)| crate::itemdata::ITEMS.get(item_kind as usize))
                .map(|d| (d.shoot_speed, d.damage.max(0), d.knock_back))
                .unwrap_or((11.5, 0, 7.0));
            // `:6416-6419` — the recycle: the lowest-`timeLeft` hook of this
            // owner dies once the total reaches `num7` (the ORIGINAL shoot's
            // budget, read before the kind cycles).
            if crate::grapple::recycles_oldest(flight, attached, original_shoot) {
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
            // `:6420-6428` — the Dual Hook's colour swap: a blue head (73)
            // already out (any state) means the new shot is the red one (74).
            if original_shoot == crate::projectile::DUAL_HOOK_BLUE {
                let blue_out = app.projectiles.slots().iter().any(|pr| {
                    pr.active && pr.owner == owner && pr.kind == crate::projectile::DUAL_HOOK_BLUE
                });
                if blue_out {
                    kind = crate::grapple::dual_next_kind(true);
                }
            }
            // `:6430-6458` — the Lunar cycle (item 3572): the highest-`timeLeft`
            // in-flight kind steps 646→647→648→649→646. The C# scan keeps the
            // FIRST slot on a `timeLeft` tie (`num9 < timeLeft`, strict), so the
            // port folds in slot order rather than `max_by_key` (last-wins).
            if item_hook.is_some_and(|(item_kind, _)| item_kind == 3572) {
                let mut best_kind: Option<u16> = None;
                let mut best_time = -1i32;
                for pr in app.projectiles.slots() {
                    if pr.active
                        && pr.owner == owner
                        && (crate::projectile::LUNAR_HOOK_SOLAR
                            ..=crate::projectile::LUNAR_HOOK_STARDUST)
                            .contains(&pr.kind)
                        && (best_time == -1 || best_time < pr.time_left)
                    {
                        best_kind = Some(pr.kind);
                        best_time = pr.time_left;
                    }
                }
                kind = crate::grapple::next_lunar_kind(best_kind);
            }
            // `:6460-6478` — the aim at the mouse world position, the velocity
            // the unit aim times `item.shootSpeed`, spawned at the player center.
            let center = Vec2::new(
                p.x + p.width * 0.5,
                p.y + p.height * 0.5,
            );
            let _spawned = app.projectiles.spawn_hook(
                kind,
                center,
                mouse_world,
                shoot_speed,
                damage,
                knock_back,
                owner,
                p.facing,
            );
            } // the `:6399` fire gate — a refused budget skips the fire tail only
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
        // `Player.controlThrow` off `Main.cThrowItem` (`Main.cs:67168`,
        // default `T` — the settings row rebinds it; the task's "Q" is a
        // rebind, not the seed). The fullscreen-map control clear
        // (`Player.cs:24801`) zeroes it while the map pans.
        app.control_throw = !map_open
            && app
                .keys
                .iter()
                .any(|k| crate::input_map::is_bound(&binds, crate::input_map::Action::Throw, *k));
        if mount_edge && !map_open {
            // `Player.TryingToUseItem` (`Player.cs:24319-24338`): the port's
            // summary — the use button held with a usable held item (the
            // `mouseInterface`/release/can-use halves fold into the editing
            // gate; the QuickMount dismounts-on-item-use refusal reads it).
            let held = app.inventory[app.selected];
            let trying_to_use_item = app.mine_down
                && !app.inventory_open
                && held.count > 0
                && crate::item::use_style(held.kind) != 0;
            if let Some(kind) =
                p.quick_mount(&app.world, &app.inventory, &app.equip.misc_equips, trying_to_use_item)
            {
                // S1-P-i54 — `QuickMount` (`Player.cs:6086-6093`) plays the
                // saddle's `UseSound` at `base.Center` after `SetMount`;
                // there is NO item animation — `ItemCheck_TryStartUse`
                // (`:52021-52040`) only gates, and `SetMount`
                // (`Mount.cs:6196-6305`) never touches itemAnimation/
                // itemTime. The port's old swing/attack_cd arming was an
                // invention and is gone. The case-2 pitch draws ride the
                // `UpdatePlayers` stream (the fire runs inside
                // `Player.Update`, `Main.cs:17754`), as the grapple's.
                if let Some((idx, pitch)) = crate::item::initial_use_sound(kind) {
                    let (cx, cy) = p.center();
                    app.audio.play_use_sound(
                        idx,
                        cx,
                        cy,
                        pitch,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                }
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
        // `:22676-22679` — the Track Hook hands the player to the equipped
        // cart mount (the misc-equip 2) when it is a cart row; the wooden
        // minecart (13) is the default. The C#'s `expertOnly` gate rides the
        // journey/expert flags the port does not carry.
        p.cart_mount = app
            .equip
            .misc_equips
            .get(2)
            .filter(|s| s.count > 0)
            .and_then(|s| crate::mount::mount_type_of_item(s.kind))
            .filter(|m| *m >= 0 && crate::mount::is_cart(*m))
            .unwrap_or(13);
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
        // The held-kind snapshot the `CanUseStressBall` grant reads
        // (`Player.cs:14352`'s `inventory[selectedItem]`), filled before the
        // tick's `update_equips` pass.
        p.held_item_kind = app.inventory[app.selected].kind;
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
            let mounted_blocks_hold = p.mount.is_active() && matches!(p.mount.kind(), 55 | 56 | 61);
            let can_visually_hold = !mounted_blocks_hold
                && (hold_style != 4 || (!p.pulley && p.vy == 0.0 && !p.mount.is_active()));
            // The hooks' centroid relative to the body centre
            // (`Player.cs:36497-36503`): the C# averages ONLY the registered
            // attached hooks (`grappling[0..grapCount]`), not every aiStyle-7
            // slot — an in-flight hook does not lean the body. Only the body
            // row leans on it.
            let mut dx = 0.0f32;
            let mut dy = 0.0f32;
            if p.grappling_active {
                let (reg, count) = app.projectiles.hook_reg();
                if count > 0 {
                    for &idx in reg[..count].iter() {
                        let pr = &app.projectiles.slots()[idx as usize];
                        dx += pr.pos.x + pr.w * 0.5;
                        dy += pr.pos.y + pr.h * 0.5;
                    }
                    dx = dx / count as f32 - (p.x + p.width * 0.5);
                    dy = dy / count as f32 - (p.y + p.height * 0.5);
                }
            }
            // `Player.cs:47371-47378`: the shooting styles' `itemRotation` —
            // `atan2(aimY * direction, aimX * direction) - fullRotation`. The
            // port keeps the aim as an angle, so the mirrored `atan2` is the
            // angle + PI when the player faces left.
            let dir = p.facing as f32;
            let item_rotation =
                crate::player::item_rotation_for_aim(app.swing_aim, p.facing, p.full_rotation);
            p.frame_inputs = crate::player::FrameInputs {
                use_style,
                // S1-P-i05: the direction lock's exemption row
                // (`Player.cs:19837`).
                use_turn: crate::registry::get(kind).is_some_and(|d| d.use_turn),
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
                swing_anim: app.swing,
                swing_anim_max: app.swing_max,
                use_animation: crate::registry::get(kind).map_or(0, |d| i32::from(d.use_animation)),
                // `Player.cs:47371-47378`: the shooting styles' `itemRotation` —
                // `atan2(aimY * direction, aimX * direction) - fullRotation`.
                // The port keeps the aim as an angle, so the mirrored
                // `atan2` is the angle + PI when the player faces left.
                item_rotation,
                // `num21 = itemRotation * direction` (`Player.cs:36423`).
                aim_relative: item_rotation * dir,
                // `!wet || !item.noWet` and `!happyFunTorchTime ||
                // createTile != 4` — the port has neither item field, so both
                // gates are open (documented in ANALYSIS).
                hold_style_dry: true,
                torch_hold_ok: true,
                grapple_dx: dx,
                grapple_dy: dy,
            };
        }
        if cart_pin && !p.cart_mounted {
            p.mount_cart();
        }
        let mut run_pixels = 0.0f32;
        app.player_step.tick(dt, |_| {
            // S1-P-i55 — `Main.LocalGolfState.Update()` (`Main.cs:17219`)
            // runs once per world update BEFORE `UpdatePlayers` — the
            // dead-player branch included (the C# call is unconditional).
            update_golf_state(
                &mut app.golf_state,
                &app.inventory,
                app.selected,
                &app.projectiles,
                app.local_id as i32,
            );
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
            // `PressurePlateHelper.PlayerLastPosition` (`PressurePlateHelper.
            // cs:16`): the player's position at the end of the last tick —
            // the port's `prev_x`/`prev_y` hold it until this tick's update
            // overwrites them.
            let plate_last = (p.prev_x, p.prev_y);
            // `Player.cs:19420` — `UpdateManaRegen`'s `flag` reads the LIVE
            // `itemAnimation`/`reuseDelay`; the port's stand-ins are the
            // App's swing/reuse timers (fed before the tick, the
            // `frame_inputs` convention).
            p.mana_regen_item_in_use = app.swing > 0.0 || app.reuse_delay > 0.0;
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
            // `Player.cs:28274-28275`: the switch-tile bridge — the player's
            // box reaching a pressure plate trips the wiring (the C#'s call
            // sits right after `FloorVisuals`, inside `Player.Update`).
            let switch = crate::collision::switch_tiles(
                &mut app.wiring,
                &mut app.world,
                p.x,
                p.y,
                p.old_x,
                p.old_y,
                p.width as i32,
                p.height as i32,
                5, // the player's objType (`Collision.SwitchTiles`)
            );
            if let Some((item, x, y)) = switch.dropped {
                // `WorldGen.KillTile`'s drop for the broken plate.
                crate::world_sim::items::spawn_item_drop_fields(
                    &mut app.items,
                    rng,
                    item,
                    1,
                    x,
                    y,
                );
            }
            // The `59 HitSwitch` send sites (`Collision.cs:3304`/`:3318`/
            // `:3324`/`:3347`).
            switch_sends.extend(switch.switches.iter().copied());
            // S1-E-21i — the tripped switches' Mech-28 clicks
            // (`Wiring.cs:267` et al.): the pitch draws ride the
            // `UpdatePlayers` stream — `Collision.SwitchTiles` runs inside
            // `Player.Update` under the `Main.cs:17754` swap.
            for (px, py) in app.wiring.take_switch_clicks() {
                app.audio.play_switch_click(
                    px as f32,
                    py as f32,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // `PressurePlateHelper.UpdatePlayerPosition(this)`
            // (`Player.cs:28276`): the weighted pressure plates (428) the
            // player stands on.
            crate::world_sim::wiring::update_pressure_plates(
                &mut app.wiring,
                &mut app.world,
                p.x,
                p.y,
                plate_last.0,
                plate_last.1,
                p.width,
                p.height,
            );
            // S1-E-21i — the weighted plates' clicks: `PressurePlateHelper.
            // Update` runs under its own `UpdatePressurePlates` swap
            // (`Main.cs:17763-17765`), so the press/release trips' pitch
            // draws ride that stream, not the player's.
            for (px, py) in app.wiring.take_switch_clicks() {
                app.audio.play_switch_click(
                    px as f32,
                    py as f32,
                    app.rngs.stream(crate::app::RngKey::UpdatePressurePlates),
                );
            }
            if run_eligible && p.vy == 0.0 {
                run_pixels += (p.x - run_x0).abs();
            }
            // `Player::update_summon_buff_arms`): the recount
            p.update_projectile_caches(&app.projectiles);
            p.update_summon_buff_arms(&mut app.projectiles, &app.inventory);
            // S1-E-7 wave C: the accessory/set-bonus buff arms
            // (`Player.cs:10800-11400` + the imbues `:12013-12044`) — the
            // same `UpdateBuffs` slot chain, in its own module so waves A/B
            // can edit the `player.rs` effect region without collision.
            crate::buff_effects_accessory::update_accessory_buff_arms(
                p,
                &mut app.projectiles,
                &app.npcs,
                // The same `UpdatePlayers` stream, re-fetched: the
                // switch-click drains above (S1-E-21i) borrowed the pool
                // after `rng`'s last use — the stream identity and the
                // draw order are unchanged.
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
            // DW-3.3/DW-3.4 — the ammo-mastery half of `Player.Update`
            // (`Player.UpdateBuffs` → `UpdateEquips` → … → `Update_AmmoCycler` →
            // `ItemCheck`, `:25566`/`:25633`/`:24566`/`:31827`), which the port
            // splits into its frame order `update_player` → `update_editing`
            // (`frame.rs:285`/`:481`): the flags clear and re-derive from this
            // tick's equipment and buffs, then the cycler's cooldown decrements
            // — both inside the fixed step, BEFORE the shoot path reads them.
            grant_ammo_mastery(p, &app.equip.armor, &app.equip.misc_equips);
            // The mechanical-accessory half of `UpdateEquips` (S1-P-i34): the
            // wire-visibility goggles + the auto-actuator flag, re-derived from
            // this tick's inventory/armor/held item (`Player.cs:12799-12820`).
            grant_mechanical_accs(
                p,
                &app.inventory,
                &app.equip.armor,
                app.inventory[app.selected].kind,
            );
            // The `UpdateEquips` summon-cap rows (`Player.cs:13254-13948`,
            // `:15324-15329`) — the armor/held half beside the ammo flags.
            grant_summon_caps(p, &app.equip.armor, &app.equip.misc_equips, &app.world);
            // `Player.cs:25709-25712` — a `maxTurrets` change drives the
            // `UpdateMaxTurrets` sweep (the lowest-`timeLeft` wipable sentry
            // dies while the budget holds); Phase 5's pool method runs it.
            // S1-P-i26: the `DD2Event.Ongoing` read is the world's live flag,
            // not the `event_ongoing()` stub (the O'A towers persist through
            // the sweep while the event runs).
            if p.max_turrets_old != p.max_turrets {
                app.projectiles.update_max_turrets(
                    p.max_turrets,
                    app.local_id as u8,
                    app.world.dd2_event_ongoing,
                );
                p.max_turrets_old = p.max_turrets;
            }
            p.ammo_cycling.tick();
            // `Player.Update_SentryBackpack` (`Player.cs:28428-28434`, called
            // at `:24568` right after `Update_AmmoCycler` `:24566`): the
            // sentry-backpack grab cooldown counts down one per tick. The
            // pickup gate (`Projectile.CanGetPickedUpBySentryBackpack`,
            // `Projectile.cs:78796`) reads it; `SetAsSelectedSentryBackpackTarget`
            // (`:78809`) re-arms it to 2 on a grab.
            if p.acc_sentry_backpack_grab_cooldown > 0 {
                p.acc_sentry_backpack_grab_cooldown -= 1;
            }
            // S1-P-i56 — `Player.Update`'s tank-pet decay
            // (`Player.cs:24539-24548`, `Player::tank_pet_decay`): while a
            // tank pet is claimed, the first tick arms the one-tick grace
            // latch; a tick where the pet did NOT re-pin it (AI_120 writes
            // `tankPetReset = false` every non-idle tick,
            // `Projectile.cs:44683`) drops the claim to -1. The projectile
            // pass runs AFTER the player pass (`frame/world.rs` players →
            // projectiles), so the previous tick's re-pin is what this
            // reads — the vanilla order. The C# runs it ahead of
            // `UpdateMinionTarget` (`:24738`), mirrored here.
            p.tank_pet_decay();
            // `Player.Update_SnakeBand` (`Player.cs:28412-28418`, called at
            // `:24567` right after `Update_AmmoCycler`): the whip-repeat
            // reset cooldown counts down one per tick; at 0 the swing
            // counter returns to −1 (the next whip throw is swing #0).
            p.update_snake_band();
            // S1-P-i27 — `Player.UpdateMinionTarget` (`Player.cs:56999-57028`):
            // the per-tick minion-target maintenance. The rest target drops
            // past 1000 px (`:57005-57007`), the attack target drops when
            // unchaseable or its HITBOX drifts past 3000 px (`:57009-57011`).
            // (Replaces the old invalidation-only half, which measured
            // centre-to-position instead of hitbox-to-centre; supersedes
            // i56's invalidate call — same halves, one canonical fn.)
            p.update_minion_target(&app.npcs);
            // S1-P-i27 — the rest-target command's input edge
            // (`Player.cs:25022-25073`, the down-cardinal double-tap/hold):
            // the Forbidden set bonus aims the rest spot on the down
            // double-tap, the Stardust/Forbidden bonuses clear it on a 60-tick
            // down hold. `Main.ReversedUpDownArmorSetBonuses` (`Main.cs:273`)
            // flips the cardinal to UP, disabling the down command.
            p.update_minion_rest_command(
                &app.world,
                mouse_world,
                input.down,
                reversed_armor_bonuses,
            );
            // `Player.cs:27608-27614`: right after the NPC collision pass,
            // the tile-contact damage check (`GetHurtTile` +
            // `ApplyTouchDamage`) — the spike hazard the dungeon pass paints,
            // the hellstone's burn, the sands' suffocation and the thorns
            // that break under a touch.
            // `Player.Update` runs under the `UpdatePlayers` swap
            // (`Main.cs:17754`): the roll rides `stream(UpdatePlayers)`.
            let touch = crate::touch_damage::apply_touch_damage(
                &mut app.world,
                p,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                p.width as i32,
                // `Player.GetHurtTile` (`Player.cs:28956`): the cart's body
                // sits 16 px higher, so the scan's box is that much shorter.
                if p.cart_mounted {
                    p.body_height() as i32 - 16
                } else {
                    p.body_height() as i32
                },
            );
            // `WorldGen.KillTile`'s item drop (`Player.cs:29013`) — the
            // thorns drop nothing in the vanilla, but the drop path is the
            // source's own (`registry::tile_drop`).
            if let Some((x, y, Some(item))) = touch.destroyed {
                // `WorldGen.KillTile`'s item drop (`Player.cs:29013`) — the
                // thorns drop nothing in the vanilla, but the drop path is the
                // source's own (`registry::tile_drop`); the roll rides the same
                // `UpdatePlayers` stream as the touch pass above.
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                crate::world_sim::items::spawn_item_drop_fields(
                    &mut app.items,
                    rng,
                    item,
                    1,
                    x,
                    y,
                );
            }
            // `Player.cs:28237-28241` + `Minecart.HitTrackSwitch`
            // (`Minecart.cs:1281-1288`): the pressure-track trip fires
            // `Wiring.HitSwitch` at the magnet tile and announces it as `59`
            // (`Minecart.cs:1287`).
            if let Some((i, j)) = p.take_switch_trip() {
                app.wiring.hit_switch(&mut app.world, i, j);
                switch_sends.push((i, j));
                // S1-E-21i — the track switch's Mech-28 click rides the
                // `UpdatePlayers` stream (`Minecart.HitTrackSwitch` runs in
                // the player tick, `Main.cs:17754`).
                for (px, py) in app.wiring.take_switch_clicks() {
                    app.audio.play_switch_click(
                        px as f32,
                        py as f32,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                }
            }
            // The wheel-spark drain: the tick queued fully-rolled dust-213
            // particles (`Minecart.WheelSparks` → `DelegateMethods.Minecart.Sparks`)
            // and the C# sound sites' cues; each request becomes the pool's
            // type-213 slot (`Main.UpdateWorld_Dust`, `Main.cs:18047`) and the
            // pool's rolls ride the client's stream (`SplitmixRand` — S1-E-1
            // phase 2).
            let sparks = p.take_cart_sparks();
            if !sparks.is_empty() {
                for s in sparks {
                    crate::minecart::spawn_spark(
                        &mut app.dust,
                        &mut crate::app::SplitmixRand(&mut p.spark_rng),
                        &spawn_ctx,
                        &app.world,
                        s,
                    );
                }
            }
            for cue in p.take_cart_sounds() {
                app.audio.play_at(cue.cue, cue.x, cue.y, 1.0);
            }
            // `Player.StickyMovement`'s handoffs: the cobweb the drag broke
            // (`Player.cs:22949` `WorldGen.KillTile`) and the type-153 web puffs
            // (`:23036-23080` `Dust.NewDust` + the `Next(0,6)`/`velocity *= 0.1`/
            // `noGravity` post-arms). The tick queues the position; the pool's
            // own rolls ride the same `UpdatePlayers` stream, so the draw order
            // is the source's (S1-A-10).
            if let Some((wx, wy)) = p.take_sticky_kill() {
                tr_core::edit::kill_tile(&mut app.world, wx, wy, false);
            }
            for puff in p.take_sticky_dust() {
                let idx = app.dust.new_dust(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &spawn_ctx,
                    &app.world,
                    puff.pos,
                    puff.w,
                    puff.h,
                    puff.kind,
                    0.0,
                    0.0,
                    if puff.post_arms { 50 } else { 0 },
                    tr_core::particles::Color::default(),
                    1.0,
                );
                if !puff.post_arms {
                    continue;
                }
                // `Player.cs:23037-23039`: the `Next(0,6)` scale, the ×0.1
                // velocity and `noGravity`.
                let slot = app.dust.slot_mut(idx);
                slot.scale += app
                    .rngs
                    .stream(crate::app::RngKey::UpdatePlayers)
                    .next_range(0, 6) as f32
                    * 0.1;
                slot.velocity *= 0.1;
                slot.no_gravity = true;
            }
            // `Player.ManaEffect` (`Player.cs:4531-4538`): the cuffs' restore
            // spawns `CombatText.NewText(playerRect, HealMana, amount)` at the
            // hurt position (`:4533`); the NetMessage 43 send (`:4534-4537`)
            // is MP-only. Same `UpdatePlayers` stream as the other player-tick
            // spawns (S1-A-10); the placement draws run inside `spawn`.
            if let Some(amount) = p.take_mana_effect() {
                let text = amount.to_string();
                let size = match app.lookups.combat_text_font.as_ref() {
                    Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
                    None => [text.chars().count() as f32 * 8.0, 10.0],
                };
                let location = crate::combat_text::Rect {
                    x: p.x,
                    y: p.y,
                    width: p.width,
                    height: p.height,
                };
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                crate::combat_text::spawn_amount(
                    &mut app.combat_texts,
                    location,
                    crate::combat_text::HEAL_MANA,
                    amount,
                    false,
                    false,
                    size,
                    1.0,
                    rng,
                );
            }
            if let Some(req) = p.take_retaliate() {
                // S1-C-7 wave B — the fall/petrify lane's retaliate block
                // (`Player.cs:38401-38517`): those hits run inside
                // `Player.Update` under the `UpdatePlayers` swap
                // (`Main.cs:17754`) on the General counter (a Retaliate
                // member), so the drain rides the same stream. The lava/tile
                // lanes pass `retaliate_lane = false` (their counters are not
                // Retaliate members) and never queue.
                let expert = p.buffs.expert_mode;
                crate::hurt_modifiers::drain_retaliate(
                    p,
                    &mut app.projectiles,
                    &mut app.npcs,
                    req,
                    app.local_id as u8,
                    expert,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // `Player.PlayHurtSound` (`Player.cs:38525`): the hurt cue plays
            // at the `Hurt` TAIL — after the cuffs' mana text (`:38291`) and
            // the retaliate block (`:38401-38517`), so its draw follows
            // theirs on the stream. The C# plays the local player's hurt
            // sound non-positionally (`Player.cs:38572`). S1-E-21f upgrades
            // the plain play to case 1 (`LegacySoundPlayer.cs:503-515`): the
            // `Main.rand.Next(3)` slot pick rides the `UpdatePlayers` stream
            // (`Player.Hurt` runs under the `Main.cs:17754` swap), only after
            // the unpositioned gate. The port has no voice setting — variant
            // 1, `voicePitchOffset` 0 (`Player.cs:38699-38704`).
            if p.take_hurt_sound().is_some() {
                app.audio.play_hurt_sound(
                    0.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            if p.take_death_sound().is_some() {
                // S1-E-21f — `Player.PlayDeathSound`'s default arm plays at
                // `base.position` (`Player.cs:38745`), the TOP-LEFT — not the
                // centre. Case 5 (`LegacySoundPlayer.cs:696-705`) draws
                // nothing; the attenuation gate still applies.
                app.audio.play_death_sound(p.x, p.y);
            }
            // W4-f3 — the petrify-transition burst (`Player.cs:25226-25233`):
            // the stoned edge queues 20 type-1 puffs + the per-puff `Next(2)`
            // noGravity roll. The C#'s `PlaySound(0)` (`:25225`) is a cited
            // seam (the stoned hurt rides `hurt_inner`'s own cue above), so the
            // drain replays only the dust chain on the `UpdatePlayers` stream.
            if let Some(burst) = p.take_stoned_dust() {
                crate::player_dust::drain_stoned_dust(app, burst);
            }
            // `Player.SpawnFastRunParticles` (`Player.cs:20214`): the
            // fast-run footstep at the player's top-left position — the
            // case-17 `Main.rand.Next(-10, 11)` pitch roll (`:862`) rides
            // the `UpdatePlayers` stream (the `Main.cs:17754` swap covers
            // `Player.Update`), only after the attenuation gate passed.
            if let Some((sound_type, style, sx, sy)) = p.take_step_sound() {
                app.audio.play_step_sound(
                    (sound_type, style),
                    sx,
                    sy,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // W4-d51 — the fast-run trail (`Player.cs:20217-20298`): the dust
            // arm follows the footstep sound on the same stream — the
            // source's sound-then-dust order inside `SpawnFastRunParticles`
            // (which runs in `HorizontalMovement`, BEFORE the jump/dash
            // chains below).
            if let Some(burst) = p.take_fast_run_dust() {
                crate::player_dust::drain_fast_run(app, burst);
            }
            // S1-E-21f — the extra-jump chain's plays: case 16's one
            // `Next(-10, 11) * 0.01` pitch (`LegacySoundPlayer.cs:851`) rides
            // the `UpdatePlayers` stream after the gate; the Fart's Item-16
            // style play rides wave A's façade (the `GetRandomPitch`
            // NextFloat on the audio-only stream — variance 0, still drawn).
            if let Some((sx, sy)) = p.take_double_jump_sound() {
                app.audio.play_double_jump_sound(
                    sx,
                    sy,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            if let Some((idx, sx, sy)) = p.take_fart_sound() {
                app.audio.play_use_sound(
                    idx,
                    sx,
                    sy,
                    0.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // W4-d51 — the extra-jump chain's dust burst (`Player.cs:
            // 20818-21012`): the tick snapshotted the box/velocity AFTER the
            // ascent write; the drain replays the arm's `Dust.NewDust` chain
            // here, right after the sound drain, so the sound-then-dust draw
            // order matches the source (the `Main.cs:17754` `UpdatePlayers`
            // swap covers both). Sandstorm / Blizzard queue no dust.
            if let Some(burst) = p.take_jump_dust() {
                crate::player_dust::drain_jump_dust(app, burst);
            }
            // W4-d51 — the `DashMovement` dust burst (`Player.cs:21259-21512`):
            // the trail (active dash) or the start burst, snapshotted at the
            // C# read moment; no sound rides the dash, so the drain follows
            // the jump chain's on the same `UpdatePlayers` stream.
            if let Some(burst) = p.take_dash_dust() {
                crate::player_dust::drain_dash_dust(app, burst);
            }
            // W4-f3 — the `DoubleJumpVisuals` trail (`Player.cs:21892-22021`):
            // the performing-jump arm's per-tick dust, snapshotted at the C#
            // read moment; the drain follows the dash dust on the same
            // `UpdatePlayers` stream (the C# runs `DoubleJumpVisuals` at
            // `:26767`, right after `DashMovement` `:26757`).
            if let Some(burst) = p.take_double_jump_dust() {
                crate::player_dust::drain_double_jump_dust(app, burst);
            }
            // S1-E-21f — the liquid entry/exit splash: case 19's playing
            // guard refuses BEFORE the one pitch draw
            // (`LegacySoundPlayer.cs:872-888`); the style picks the slot and
            // the shimmer ×0.75 cut. Same stream, after the gate.
            if let Some((style, sx, sy)) = p.take_splash_sound() {
                app.audio.play_splash_sound(
                    style,
                    sx,
                    sy,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // S1-E-21f — the drowning gasp (`Player.cs:23257`): case 23 is
            // Volume/Pan only — no draws, no stream.
            if p.take_drown_sound() {
                app.audio.play_drown_sound();
            }
            // W4-f3 — the drowning bubble (`Player.cs:23283-23316`): the
            // `Next(20)` gate drew on the tick side; the type-34 puff rides the
            // pool's own wet-probe kill. Same stream, after the gasp.
            if let Some((mouth, width)) = p.take_drown_bubble() {
                crate::player_dust::drain_drown_bubble(app, mouth, width);
            }
            // W4-f3 — the floor dust (`Player.cs:23790-24043`, `MakeFloorDust`):
            // the `FloorVisuals` call (`:28271`) queues the floor-tile snapshot;
            // the drain replays the per-iteration gate rolls + the NewDust chain
            // on the `UpdatePlayers` stream (the shimmer orchestrator + Item177
            // cue are cited seams).
            if let Some(burst) = p.take_floor_dust() {
                crate::player_dust::drain_floor_dust(app, burst);
            }
            // W4-f3 — the wing-glide dust (`Player.cs:27040-27337`): the glide
            // arm queues the per-wing snapshot; the drain replays the gate rolls
            // + the NewDust chains on the `UpdatePlayers` stream (the wings-4
            // wingFrame machine + Item13 cadence are cited seams).
            if let Some(burst) = p.take_wing_glide_dust() {
                crate::player_dust::drain_wing_glide(app, burst);
            }
            // S1-E-21f — the wing flap's Item-32 play (`Player.cs:26850`):
            // wave A's façade, the same stream contract as the Fart.
            if let Some((idx, sx, sy)) = p.take_flap_sound() {
                app.audio.play_use_sound(
                    idx,
                    sx,
                    sy,
                    0.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // S1-P-i41 — the Chillemt mounts' equip cue (`Mount.cs:6285`):
            // the case-42 trackable façade, the NextFloat on the audio-only
            // stream (the `play_chillet_sound` contract).
            if let Some((sx, sy)) = p.take_chillet_sound() {
                app.audio.play_chillet_sound(sx, sy);
            }
            // S1-P-i41 — the QuickMount no-space tink (`Player.cs:6057`):
            // case 0's Dig variation + pitch draws ride the `UpdatePlayers`
            // stream (the player tick runs under the `Main.cs:17754` swap).
            if let Some((sx, sy)) = p.take_no_space_sound() {
                app.audio.play_no_space_sound(
                    sx,
                    sy,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // S1-P-i41 — the third-fail no-space teleport (`Player.cs:6061-
            // 6066`): the forced dismount ran inside `quick_mount`, the spot
            // scan + teleport need `&World` (the `pending_dismount` convention).
            if p.take_no_space_teleport() {
                p.player_no_space_teleport(&app.world);
                // `Shellphone_Spawn`'s fallback may queue the force-clear.
                force_clear = p.take_force_clear();
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
            // S1-P-i14 — the Revolver 2269's per-tick crit charge/decay
            // (`Player.ItemCheck`, `Player.cs:42677-42703`). The C# runs it
            // EVERY tick for EVERY held item, inside `ItemCheck` (`:31827`,
            // after the movement/hurt/buff draws above) under the
            // `UpdatePlayers` swap — the i01 census's determinism P0: the
            // `Main.rand.Next(3)` draw was MISSING from the port, so every
            // downstream draw sat one roll early.
            //
            // Holding the Revolver with the button RELEASED mid-swing
            // (`itemAnimation > 0 && !controlUseItem`, `:42679`) charges the
            // bonus +1 and cancels the swing one tick early (the extra
            // `itemAnimation--`/`itemTime--`, `:42681-42690`) — NO draw on that
            // path. Otherwise a 1/3 `Next(3)` roll decays the bonus −2
            // (`:42693-42696` for the Revolver, `:42700-42703` for EVERY other
            // item — the else arm draws unconditionally). The Revolver then
            // clamps `item.crit + bonus` to `[crt, 20 + crt]` (`:42697-42698`,
            // `crt` = the prefix's additive crit). The shot reads the bonus
            // through the throw-time crit roll (`world_sim/editing.rs`,
            // `Projectile.cs:11129-11132`).
            //
            // `ItemCheck` returns early when `CCed` (`Player.cs:42598-42602`:
            // frozen/webbed/stoned), so the whole block — draw included — skips
            // on a CCed tick. The dead player never reaches `ItemCheck` either
            // (`Player.Update`'s `UpdateDead` return, `:24749-24754`); the
            // port's dead path returns above, so this closure is alive-only.
            if !p.cced() {
                let held = app.inventory[app.selected];
                // The C# compares `itemAnimation > 0` as an INTEGER
                // (`:42679`); the port derives the tick count the same way the
                // shooter wire does (`ticks_of`, `world_sim/editing.rs`).
                let anim_ticks = (app.swing * 60.0).round() as i32;
                let revolver_charging =
                    held.kind == 2269 && anim_ticks > 0 && !app.mine_down;
                if revolver_charging {
                    // The cancelled-swing decrement (`:42681-42690`): one extra
                    // tick off both timers, clamped at zero. The port's timers
                    // are seconds decaying per frame; one 60 Hz tick is `dt`.
                    app.swing = (app.swing - dt).max(0.0);
                    app.item_time = (app.item_time - dt).max(0.0);
                    p.revolver_crit_chance_bonus += 1;
                } else if app.rngs.stream(crate::app::RngKey::UpdatePlayers).next_max(3) == 0 {
                    p.revolver_crit_chance_bonus -= 2;
                }
                if held.kind == 2269 {
                    // `Utils.Clamp(item.crit + bonus, crt, 20 + crt) - item.crit`
                    // (`:42698`): `item.crit` is the prefixed instance crit
                    // (`Item.Prefix`, `Item.cs:584` `crit += crt`), `crt` the
                    // prefix's additive bonus alone.
                    let crt = crate::prefix::prefix_crit_bonus(held.prefix);
                    let item_crit = crate::prefix::prefixed_stats(2269, held.prefix)
                        .map_or(0, |s| s.crit);
                    p.revolver_crit_chance_bonus =
                        (item_crit + p.revolver_crit_chance_bonus).clamp(crt, 20 + crt)
                            - item_crit;
                }
            }
        });
        app.player_step.drop_backlog();
        if run_pixels > 0.0 {
            // `Player.cs:28233-28236` -> `AchievementsHelper.HandleRunning`
            // (`AchievementsHelper.cs:420-423`): grounded, unmounted running
            // advances MARATHON_MEDALIST's `Move` float accumulator.
            let _ = app.achievements.notify_custom_float("Move", run_pixels);
        }
        // `RemoveAllGrapplingHooks` (`Player.cs:33562-33574`): the jump-off
        // kills every aiStyle-7 slot this player owns, clears the owner's
        // refused-tile set and drops the registration — the pool helper carries
        // the source's `ClearGrapplingBlacklist` + `grappling[0] = -1;
        // grapCount = 0` head (`:33563-33565`) beside the `for (i < 1000)` kill.
        //
        // The C# calls the removal from the jump-off (`:22733`), the sign
        // interaction (`:33379`), the minecart-hook launch (`:33557`) and the
        // TrackHook→cart transfer (`:22675`) — not from "the player is not
        // grappling", so a freshly fired hook survives its own tick.
        if p.release_grappling_hooks {
            p.release_grappling_hooks = false;
            let owner = app.local_id as u8;
            app.projectiles.remove_all_grappling_hooks(owner);
        }
        if app.follow_player {
            // fix/player-motion-blur: the follow target is the RAW fixed-step
            // position — the C# camera reads `LocalPlayer.Bottom + (0, gfxOffY
            // - 21)` (`PlayerFocusedScreenPosition`, `Main.cs:62567-62570`),
            // never a between-tick blend (the old render blend had
            // no C# counterpart). The draw-time snap/truncate/clamp lives in
            // `update_camera_at_draw` (`DoDraw_UpdateCameraPosition`,
            // `Main.cs:62590-62649`); this update-tail ease only shapes the
            // mid-tick camera reads (aiming, `mouse_world`).
            let (cx, cy) = p.center();
            app.camera = app.camera.lerp(Vec2::new(cx, cy), 1.0 - (-dt * 12.0).exp());
        }
    }
    // The `59 HitSwitch` sends (`Collision.cs:3304`/`:3318`/`:3324`/`:3347`) —
    // after the tick closure and the player borrow released `app`.
    for (x, y) in switch_sends {
        app.send_hit_switch(x, y);
    }
    // `Spawn_ForceClearArea` (`Player.cs:37914-37935`): the same-tick
    // deferral is observation-identical — nothing probes the spawn area
    // between the teleport and this drain.
    if let Some((fx, fy)) = force_clear {
        super::editing::spawn_force_clear_area(app, fx, fy);
    }

    // `Player.cs:25214-25217` — `dropItemCheck` runs when `itemAnimation ==
    // 0 && ItemTimeIsZero && reuseDelay == 0` (the port's swing/item_time/
    // reuse_delay seconds trio). Per-frame like the hook/mount key edges
    // above (the borrow-free tail); the throw key + the cursor-stack arms
    // live in `world_sim/items.rs::drop_item_check`. The right-click arm is
    // the pre-pass press edge outside every slot (`drop_right_edge`, the
    // `mouseInterface` stand-in the players phase captures).
    if app.swing <= 0.0 && app.item_time <= 0.0 && app.reuse_delay <= 0.0 {
        let right_click_drop = app.drop_right_edge;
        crate::world_sim::items::drop_item_check(app, right_click_drop);
    }
    // `Player.cs:25248-25255` — the releaseThrow latch refresh at the tick
    // tail: held key → false, released → true. The check above read LAST
    // tick's value, so the edge fires once.
    // `Player.cs:43338-43339` — `JustDroppedAnItem` clears when the drop
    // animation (`SetItemAnimation(10)`) runs out.
    if let Some(p) = app.player.as_mut() {
        p.release_throw = !app.control_throw;
        if app.swing <= 0.0 {
            p.just_dropped_an_item = false;
        }
    }
    crate::chest::tick_open(app); // `Player.HandleBeingInChestRange` (`Player.cs:25290`/:31015-31124)
    if map_open {
        app.update_map_input();
    }
}

/// S1-P-i54 — `Player.QuickHeal`'s H-key edge (`Player.cs:5501-5547`): the
/// drink itself is the S1-P-i37 canonical `potions::quick_heal` (the
/// keep-one reconciliation, gate V4) — the gate chain (`:5503`), the
/// chooser (`QuickHeal_GetItemToUse`, `:5549-5566`), the
/// `ItemCheck_TryStartUse` gate, the early dismount (`:5512-5515`), the
/// UseSound at `base.position` (`:5516`), the per-kind potion delay with
/// the live pStone scaling (`ApplyPotionDelay`, `:51853-51886`), the heal
/// (`ApplyLifeAndOrMana`, `:51888-51952`), the hunger reset, the buff
/// grant, the stack decrement (`:5535-5542`) and the Bottled Water
/// achievement all live there. NO item animation: `ItemCheck_TryStartUse`
/// (`:52021-52040`) only gates, it never sets itemAnimation/itemTime.
///
/// RNG order (`UpdatePlayers` stream — the call runs inside
/// `Player.Update`, `Main.cs:17754`): the UseSound's case-2 pitch draws
/// FIRST (`:5516` precedes `:5519-5521`), then the Strange Brew (3001)
/// delay roll `Next(2400, 4201)` (`:51857`), then its heal roll
/// `Next(healLife, 121)` (`:51895`), then the i-frame `NextFloat`
/// (`:51898`), then the HealEffect's two position-jitter draws
/// (`CombatText.cs:88-89`, via `:51931`). Every other potion row draws
/// only the sound + the HealEffect jitter.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::inv::EquipSlots;
    use crate::item::EMPTY_SLOT;
    use crate::player::Player;
    use crate::touch_damage::BUFF_BLEEDING;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::wiring::WireColor;
    use tr_core::{Tile, UnifiedRandom, World};

    /// DW-4.1 (the players group) + DW-4.4: the touch-damage rolls draw the
    /// `UpdatePlayers` stream — one `update_player` tick standing on spikes
    /// spends `DamageVar`'s draw and then the landing-hit's bleeding duration
    /// `Next(600, 1200)` (`Player.cs:28982` + `:28986-28994` — the S1-E-13a
    /// burning/bleeding arm rides the same `Main.rand`, here the
    /// `UpdatePlayers` swap, `Player.cs:27608-27614`, `Main.cs:17754`), the
    /// landed HP and the buff duration match the twin replay, and no other
    /// stage stream moves (the pass's isolation).
    #[test]
    fn test_dw_4_1_the_touch_roll_draws_the_update_players_stream() {
        let mut app = flow_app(None, vec![], vec![]);
        // The touch_damage.rs rig shape: a deep world with a 3x3 spike block
        // the player's box stands in.
        app.world = World::new("spikes".into(), 40, 200);
        app.world.compute_column_surface();
        for x in 10..13 {
            for y in 30..33 {
                let i = app.world.index(x, y);
                app.world.tiles[i] = Tile {
                    kind: crate::touch_damage::SPIKE,
                    ..Default::default()
                };
                app.world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut p = Player::spawn(&app.world);
        p.x = 11.0 * 16.0;
        p.y = 30.0 * 16.0 - 30.0;
        app.player = Some(p);
        // Every stream starts pristine at the world seed (flow_app's seed 0).
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} pristine",
                key.name()
            );
        }
        let hp_0 = app.player.as_ref().unwrap().hp;
        update_player(&mut app, 1.0 / 60.0);

        // The twin: `DamageVar`'s draw, then the bleeding arm's duration roll
        // (both off the pristine seed-0 stream).
        let mut expect = UnifiedRandom::new(0);
        let expected_dmg = tr_core::damage::damage_var(&mut expect, 60.0, 0.0);
        let expected_bleed = expect.next_range(600, 1200);
        // S1-E-21f — the hurt tail's case-1 slot pick (`LegacySoundPlayer
        // .cs:505`) rides the same stream AFTER the damage draws (the
        // `Hurt` tail runs last).
        let _expected_hit_slot = expect.next_range(0, 3);
        // S1-P-i14 — the Revolver per-tick `Next(3)` decay draw
        // (`Player.cs:42700-42703`) runs in `ItemCheck` AFTER the hurt draws,
        // for EVERY held item (the empty hand included). It was MISSING from
        // the port (the i01 determinism P0); the stream now carries it.
        let _expected_revolver_decay = expect.next_max(3);
        assert_eq!(
            app.player.as_ref().unwrap().hp,
            hp_0 - expected_dmg,
            "the spike's rolled 60 landed through the same draws"
        );
        let slot = app
            .player
            .as_ref()
            .unwrap()
            .buffs
            .find_buff_index(BUFF_BLEEDING)
            .expect("the spike's bleeding landed");
        assert_eq!(
            app.player.as_ref().unwrap().buffs.buff_time[slot],
            expected_bleed,
            "the bleeding duration is the same stream's second draw"
        );
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            expect,
            "the DamageVar + bleeding + hurt-slot draws on UpdatePlayers"
        );
        // The isolation: no other stage stream moved.
        for key in RngKey::ALL {
            if key == RngKey::UpdatePlayers {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
    }

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

    /// S1-P-i53 — the buff-bar cancel drains for a LIVING player: the drain
    /// used to sit inside the `p.dead` branch, so an alive player's removal
    /// requests were never applied (`Main.DrawBuffIcon`'s cancel,
    /// `Main.cs:43346-43349`, has no dead gate). A queued non-debuff slot
    /// dels on the tick; a queued debuff refuses and stays.
    #[test]
    fn s1_p_i53_the_alive_player_drains_the_buff_cancel_requests() {
        let mut app = ammo_app();
        let p = app.player.as_mut().unwrap();
        assert!(!p.dead);
        p.buffs.add_buff(5, 3600); // Ironskin — removable
        p.buffs.add_buff(20, 3600); // Poisoned — a debuff
        let iron = p.buffs.find_buff_index(5).unwrap();
        let poison = p.buffs.find_buff_index(20).unwrap();
        app.buff_removals.push(poison);
        app.buff_removals.push(iron);
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.buffs.find_buff_index(5), None, "the alive cancel landed");
        assert!(
            p.buffs.find_buff_index(20).is_some(),
            "the debuff refusal (`Main.cs:43373`)"
        );
        assert!(app.buff_removals.is_empty(), "the queue drained");
    }

    /// S1-P-i53 — the drain feeds `miscEquips[0..2].buffType` into the
    /// `TryRemovingBuff_CheckBuffHideMisc` arms (`Main.cs:43380-43381`,
    /// `:43390-43404`): cancelling the equipped vanity pet's buff sets
    /// `hideMisc[0]`. The pet item 603 (Carrot) grants buff 40 (the registry
    /// row's `buff_type`; `Item.cs`'s case 603).
    #[test]
    fn s1_p_i53_the_drain_hides_the_equipped_pet_on_cancel() {
        let mut app = ammo_app();
        let pet_buff = crate::registry::get(603).unwrap().buff_type;
        assert!(pet_buff > 0, "the registry carries the pet's buff");
        app.equip.misc_equips[0] = ItemSlot {
            kind: 603,
            name: crate::registry::name(603),
            count: 1,
            prefix: 0,
        };
        let p = app.player.as_mut().unwrap();
        p.buffs.add_buff(pet_buff, 3600);
        let slot = p.buffs.find_buff_index(pet_buff).unwrap();
        app.buff_removals.push(slot);
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(p.hide_misc[0], "the pet visual hid (`:43401`)");
        assert!(!p.hide_misc[1]);
        assert_eq!(p.buffs.find_buff_index(pet_buff), None);
    }

    /// S1-P-i53 — the cancel draws NO RNG: every arm is state-only
    /// (`TryRemovingBuff`, `DelBuff`, `CheckBuffHideMisc` — none draws), so
    /// a tick with a queued Ironskin cancel leaves every stage stream at
    /// the same position as the twin tick with no queue (Ironskin grants
    /// defense only — no draw-bearing behaviour differs between the twins).
    #[test]
    fn s1_p_i53_the_buff_cancel_draws_no_rng() {
        let mut app = ammo_app();
        let mut twin = ammo_app();
        for a in [&mut app, &mut twin] {
            a.player.as_mut().unwrap().buffs.add_buff(5, 3600);
        }
        let slot = app.player.as_ref().unwrap().buffs.find_buff_index(5).unwrap();
        app.buff_removals.push(slot);
        tick_ammo(&mut app);
        update_player(&mut twin, 1.0 / 60.0);
        assert_eq!(app.player.as_ref().unwrap().buffs.find_buff_index(5), None);
        for key in RngKey::ALL {
            assert_eq!(
                *app.rngs.stream(key),
                *twin.rngs.stream(key),
                "{} — the cancel added no draws",
                key.name()
            );
        }
    }

    /// S1-E-34 — the F3 debug god mode rides `debug_god_mode` and the tick
    /// ORs it into `creative_god_mode`, so `Hurt`'s first gate
    /// (`Player.cs:38203-38206`) swallows the hit even with the journey power
    /// off. The two flags stay independent: the power's own toggle still
    /// drives its own half.
    #[test]
    fn s1_e_34_the_debug_god_mode_swallows_the_hit_with_the_power_off() {
        let mut app = ammo_app();
        app.player.as_mut().unwrap().debug_god_mode = true;
        let p = app.player.as_mut().unwrap();
        p.hp = 5;
        p.mana = 0;
        p.breath = 0;
        p.hurt_cd = 0.0;
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(
            p.creative_god_mode,
            "the tick ORs the debug flag onto the power flag"
        );
        assert_eq!(p.hp, p.max_hp, "the pool refill still runs");
        let p = app.player.as_mut().unwrap();
        p.hurt_cd = 0.0;
        assert!(
            !p.hurt(10, 0.0, &mut tr_core::UnifiedRandom::default(), true),
            "the god-mode gate swallows the hit"
        );

        // Off again: the next tick clears the flag and the hit path is live.
        app.player.as_mut().unwrap().debug_god_mode = false;
        tick_ammo(&mut app);
        assert!(!app.player.as_ref().unwrap().creative_god_mode);
    }

    #[test]
    fn s1_c_7_the_cuffs_mana_text_drains_into_the_combat_pool() {
        // `Player.ManaEffect` (`Player.cs:4531-4538`) — the cuffs' queued
        // amount spawns one `CombatText.NewText(playerRect, HealMana, amount)`
        // (`:4533`) with the placement draws on the `UpdatePlayers` stream.
        let mut app = ammo_app();
        // The tick copies `app.equip.armor` into `p.equipped` (`Player.cs:12892`).
        app.equip.armor[3].kind = 1595;
        tick_ammo(&mut app); // the fold arms magicCuffs
        assert!(app.player.as_ref().unwrap().magic_cuffs);
        {
            let p = app.player.as_mut().unwrap();
            p.mana = 0;
            p.mana_max = 100;
            p.hurt_cd = 0.0;
            p.immune = false;
            assert!(p.hurt(20, p.x + 100.0, &mut UnifiedRandom::new(0), true));
        }
        let before = app.rngs.stream(RngKey::UpdatePlayers).clone();
        tick_ammo(&mut app);
        assert_eq!(app.player.as_ref().unwrap().mana, 20);
        let t = app
            .combat_texts
            .iter()
            .find(|t| t.active)
            .expect("the HealMana text spawned");
        assert_eq!(t.text, "20");
        assert_eq!(t.color, crate::combat_text::HEAL_MANA);
        assert!(!t.crit && !t.dot);
        // The spawn's two placement `Next` rolls (`CombatText.cs:88-89`) drew
        // the player-tick stream — nothing else moved.
        assert_ne!(*app.rngs.stream(RngKey::UpdatePlayers), before);
        for key in RngKey::ALL {
            if key == RngKey::UpdatePlayers {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} stayed pristine",
                key.name()
            );
        }
        // A one-shot: the next tick without a new hurt adds no text.
        tick_ammo(&mut app);
        assert_eq!(
            app.combat_texts.iter().filter(|t| t.active).count(),
            1,
            "the queued amount drained exactly once"
        );
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
        assert!(
            p.hurt(10, 0.0, &mut tr_core::UnifiedRandom::default(), true),
            "the hurt lands once the power is off"
        );
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

    /// S1-P-i34 — the mechanical-accessory grant seam
    /// (`RefreshMechanicalAccsFromItemType`, `Player.cs:12799-12820`): the
    /// Mechanical Lens (3619) / Wire Kite (3611) grant `InfoAccMechShowWires`,
    /// the Actuation Accessory (3624) grants `autoActuator`; the per-tick
    /// `ResetEffects` clear (`:18524`/`:18534`) wipes them once unequipped.
    #[test]
    fn the_mechanical_accessories_grant_the_wire_flags_and_clear_per_tick() {
        let mut app = ammo_app();
        // The lens in an accessory slot grants the wire-visibility flag.
        app.equip.armor[3] = slot(3619);
        tick_ammo(&mut app);
        assert!(
            app.player.as_ref().unwrap().info_acc_mech_show_wires,
            "3619 Mechanical Lens shows the wires"
        );
        // Unequip: the clear wipes it.
        app.equip.armor[3] = EMPTY_SLOT;
        tick_ammo(&mut app);
        assert!(
            !app.player.as_ref().unwrap().info_acc_mech_show_wires,
            "the seam clears per tick"
        );
        // The Wire Kite (3611) grants the same flag.
        app.equip.armor[4] = slot(3611);
        tick_ammo(&mut app);
        assert!(app.player.as_ref().unwrap().info_acc_mech_show_wires, "3611 Wire Kite");
        app.equip.armor[4] = EMPTY_SLOT;
        // The Actuation Accessory (3624) in an accessory slot grants autoActuator.
        app.equip.armor[5] = slot(3624);
        tick_ammo(&mut app);
        assert!(app.player.as_ref().unwrap().auto_actuator, "3624 equipped");
        app.equip.armor[5] = EMPTY_SLOT;
        tick_ammo(&mut app);
        assert!(!app.player.as_ref().unwrap().auto_actuator, "cleared");
        // The held-item arm (`ApplyEquipFunctional`, `:15295-15298`): 3624 in
        // hand also grants autoActuator.
        app.inventory[app.selected] = slot(3624);
        tick_ammo(&mut app);
        assert!(
            app.player.as_ref().unwrap().auto_actuator,
            "3624 held grants autoActuator"
        );
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

    /// S1-L-10a — `Player.QuickGrapple_GetItemToUse` (`Player.cs:6481-6500`):
    /// the hook fires with the kind the held item's `shoot` points at (the
    /// Ivy Whip 185 shoots the Ivy Whip hook 32, not the default Hook 13).
    #[test]
    fn the_quick_grapple_uses_the_hook_item() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        app.inventory[0] = crate::item::ItemSlot {
            kind: 185,
            name: "Ivy Whip",
            count: 1,
            prefix: 0,
        };
        // The port's extra quick-grapple key (`G`) — the default `cHook`
        // bind is not in the fixture's keymap.
        app.keys.insert(KeyCode::KeyG);
        update_player(&mut app, 1.0 / 60.0);
        let kinds: Vec<u16> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active && p.owner == app.local_id as u8)
            .map(|p| p.kind)
            .collect();
        assert_eq!(kinds, vec![32], "the item's `shoot` picks the hook kind");
    }

    /// S1-E-21f — `QuickGrapple`'s fire sound (`Player.cs:6408`):
    /// `PlaySound(item.UseSound, base.position)` plays the hook item's own
    /// use-sound row through the wave-A façade on `UpdatePlayers`, once per
    /// fire; the TR_HOOK pin (no granting item) stays silent.
    #[test]
    fn s1_e_21f_the_grapple_fire_plays_the_items_use_sound() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        app.inventory[0] = crate::item::ItemSlot {
            kind: 84, // the Grappling Hook — UseSound = SoundID.Item1
            name: "Grappling Hook",
            count: 1,
            prefix: 0,
        };
        app.keys.insert(KeyCode::KeyG);
        let before = app.audio.played_count();
        let stream_before = app.rngs.stream(RngKey::UpdatePlayers).clone();
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.audio.played_count(), before + 1, "the fire chime, once");
        assert_ne!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            stream_before,
            "the case-2 draws ride UpdatePlayers"
        );
        // The held-key tick does not re-fire (the releaseHook latch).
        let mid = app.audio.played_count();
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.audio.played_count(), mid, "no re-fire while held");
        // No granting item: the pin fires the hook but plays nothing.
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        app.keys.insert(KeyCode::KeyG);
        let before = app.audio.played_count();
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.audio.played_count(), before, "no item, no UseSound");
    }

    /// S1-L-10c — `Player.QuickGrapple` (`Player.cs:6334-6337`): the mount
    /// dismounts before the hook fires only when it is outside
    /// `MountID.Sets.CanUseHooks` ({54, 57-60}) — the port dismounted every
    /// mount.
    #[test]
    fn the_quick_grapple_keeps_a_mount_inside_can_use_hooks() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        // The row 54 is in the set.
        app.player.as_mut().unwrap().set_mount(54, false);
        app.keys.insert(KeyCode::KeyG);
        update_player(&mut app, 1.0 / 60.0);
        assert!(
            app.player.as_ref().unwrap().mount.is_active(),
            "the set's mount stays mounted"
        );
    }

    /// S1-L-10c — the other side of the same gate: a mount outside the set
    /// dismounts when the hook fires.
    #[test]
    fn the_quick_grapple_dismounts_a_mount_outside_can_use_hooks() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        // The Unicorn (10) is not in the set.
        app.player.as_mut().unwrap().set_mount(10, false);
        app.keys.insert(KeyCode::KeyG);
        update_player(&mut app, 1.0 / 60.0);
        assert!(
            !app.player.as_ref().unwrap().mount.is_active(),
            "the mount dismounted"
        );
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
    /// in the App's shared `particles::DustPool` (the `Main.dust` analog) and the
    /// queued cues go out through `app.audio.play_at` (a silent no-op without
    /// an output device) — both pendings end up empty.
    /// The rig body: `seed` seeds the spark stream, the camera rides the
    /// player (the pool's C# cull spans the view, `Dust.cs:97-101`), and the
    /// pool is reset after the settle ticks so the braking burst's landings
    /// count alone.
    fn braking_pool_count(seed: u64) -> usize {
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
        p.spark_rng = seed;
        // The camera rides the player: the pool's C# cull spans the view
        // (`Dust.cs:97-101`), so every drain's spawn passes it.
        app.camera = Vec2::new(p.x + 10.0, p.y + 26.0);
        app.player = Some(p);
        // The settle tick brings the previous tick's `onTrack` up.
        for _ in 0..2 {
            update_player(&mut app, 1.0 / 60.0);
        }
        // The settle ticks drained their own rolling sparks into the pool —
        // start the burst count clean.
        app.dust = tr_core::particles::DustPool::new();
        // Brake: hold left.
        app.keys.insert(KeyCode::KeyA);
        update_player(&mut app, 1.0 / 60.0);
        crate::minecart::live_type(&app.dust, 213)
    }

    #[test]
    fn update_player_drains_cart_sparks_into_the_pool() {
        // A stream seed whose braking tick lands 3 sparks — the seed search
        // now runs the whole tick+drain pipeline, because the pool's spawn
        // rolls ride the same stream (S1-E-1 phase 2) and the settle ticks'
        // landed bursts consume pool draws ahead of the brake.
        let seed = (1u64..)
            .find(|&s| braking_pool_count(s) == 3)
            .expect("a seed braking into 3 sparks");
        assert_eq!(braking_pool_count(seed), 3);
    }

    /// Finds a stream seed where one rolling call at `|vx| = vel_x` queues
    /// exactly `want` sparks — the gate rolls interleave with the per-spark
    /// request rolls (the rain.rs seed-search idiom, run against the same
    /// emitter constants the tick passes).
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

    // ------------------------------------------------------------------
    // S1-P-i54 — hotbar/quick-switch witnesses (written, not run — the
    // code-only swarm has no compiler). Every assert cites the C# line.
    // ------------------------------------------------------------------

    /// A headless App with a live player — the switch witnesses' rig.
    fn switch_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        app
    }

    fn stocked(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: "",
            count,
            prefix: 0,
        }
    }

    #[test]
    fn s1_p_i54_a_digit_buffers_and_the_idle_tick_applies_it() {
        // `Player.cs:25095-25134` (the digit latches num7) + `:25174-25177`
        // (`Select`) + `:25218` (`Update` applies when idle).
        let mut app = switch_app();
        app.inventory[3] = stocked(9, 5);
        app.world_key(KeyCode::Digit4);
        assert_eq!(app.selected, 0, "the press only buffers");
        assert_eq!(app.selection.buffered, 3);
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.selected, 3, "the idle tick applies the buffer");
        assert_eq!(app.selection.buffered, -1);
    }

    #[test]
    fn s1_p_i54_a_mid_swing_switch_defers_until_the_swing_ends() {
        // `Player.cs:428-438` — `CanChangeSelectedItemImmediately` is false
        // while `UsingOrReusingItem` (`:4068-4078`: itemAnimation > 0).
        let mut app = switch_app();
        app.inventory[1] = stocked(9, 5);
        app.swing = 0.5; // a live swing
        app.swing_max = 0.5;
        app.world_key(KeyCode::Digit2);
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.selected, 0, "the swing blocks the switch");
        assert_eq!(app.selection.buffered, 1, "the request stays armed");
        // The swing decays to 0 (the editing pass owns the decay; the
        // witness clears it directly) — the next tick applies.
        app.swing = 0.0;
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn s1_p_i54_a_buffered_switch_interrupts_the_channel_the_same_tick() {
        // `Player.cs:42800-42807` — `HasBufferedChange` forces flag5 false
        // and `!flag5` clears `channel` the SAME tick; the switch itself
        // lands the next tick (`CanChangeSelectedItemImmediately` reads
        // the now-off channel, `:4072`). The interrupt clears MAIN's
        // channel seam (channel/channel_item/channel_target), not the
        // i20 branch's future channel_shoot field.
        let mut app = switch_app();
        app.inventory[2] = stocked(4, 1); // the Iron Broadsword — non-channel
        {
            let p = app.player.as_mut().unwrap();
            p.channel = true;
            p.channel_item = 113; // the Magic Missile's kind
            p.channel_target = Vec2::new(500.0, 400.0);
        }
        app.world_key(KeyCode::Digit3);
        update_player(&mut app, 1.0 / 60.0);
        {
            let p = app.player.as_ref().unwrap();
            assert!(!p.channel, "the buffered switch dropped the channel");
            assert_eq!(p.channel_item, 0);
            assert_eq!(p.channel_target, Vec2::ZERO);
        }
        assert_eq!(app.selected, 0, "the switch itself waits one tick");
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.selected, 2, "the switch lands once idle");
        assert_eq!(app.item_time, 0.0, "OnSelectionChanged :553");
    }

    #[test]
    fn s1_p_i54_the_wheel_cycles_the_hotbar_latch_backward() {
        // `Player.cs:31790` (`ScrollWheelDelta / -120`: wheel-up steps
        // BACK) + `:31794` (the cycle base is the latched Hotbar, not the
        // live/overridden slot).
        let mut app = switch_app();
        app.selection.hotbar = 7;
        app.selected = 2; // an overridden live slot
        app.selection.overridden = 7;
        // One wheel notch UP (step +1): 7 - 1 = 6, NOT 2 + 1 = 3.
        let target = crate::hotbar::clamp_hotbar_offset(app.selection.hotbar as i32 - 1);
        assert!(app.selection.select(app.selected, target, false));
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.selected, 6);
        assert_eq!(app.selection.hotbar, 6);
        // The wrap: notch up from 0 lands on 9 (`ClampHotbarOffset`,
        // `:31895-31906`).
        app.selection.hotbar = 0;
        let target = crate::hotbar::clamp_hotbar_offset(-1);
        assert!(app.selection.select(app.selected, target, false));
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.selected, 9);
    }

    #[test]
    fn s1_p_i54_smart_select_overrides_to_the_pickaxe_and_restores() {
        // `Player.cs:17417-17419` (the override arm) + `:17434-17460` (the
        // lookup) + `:464-484` (the restore). LeftShift (`cTorch`) held
        // over a stone tile picks the pickaxe; releasing restores.
        let mut app = switch_app();
        app.world = World::new("ss".into(), 40, 40);
        let i = app.world.index(11, 10);
        app.world.tiles[i] = Tile {
            kind: 1, // stone
            ..Default::default()
        };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        let mut p = Player::spawn(&app.world);
        p.x = 160.0;
        p.y = 160.0;
        app.player = Some(p);
        app.inventory[5] = stocked(1, 1); // the Iron Pickaxe
        // The mouse over tile (11, 10): camera at the player centre, the
        // deterministic 1280x720 screen, zoom 1.
        app.camera = Vec2::new(170.0, 181.0);
        app.mouse_pos = Vec2::new(
            640.0 + (11.0 * 16.0 + 8.0 - 170.0),
            360.0 + (10.0 * 16.0 + 8.0 - 181.0),
        );
        app.keys.insert(KeyCode::ShiftLeft);
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.selected, 5, "the pickaxe overrides");
        assert_eq!(app.selection.overridden, 0, "the old slot stored");
        // Release LeftShift: the restore lands.
        app.keys.remove(&KeyCode::ShiftLeft);
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.selected, 0, "the restore");
        assert_eq!(app.selection.overridden, -1);
    }

    #[test]
    fn s1_p_i54_quick_heal_drinks_the_best_potion_and_arms_the_sickness() {
        // `Player.cs:5501-5547`: the H press heals, decrements, arms buff
        // 21 at 3600 (`Item.potionDelay`, `Item.cs:49968`) and spawns the
        // green number; NO swing (ItemCheck_TryStartUse only gates).
        let mut app = switch_app();
        app.inventory[4] = stocked(188, 3); // the Healing Potion, heal 100
        {
            let p = app.player.as_mut().unwrap();
            p.hp = 50;
            p.max_hp = 200;
        }
        app.keys.insert(KeyCode::KeyH);
        update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.hp, 150, "the 100 heal landed");
        let sick = p.buffs.find_buff_index(21).expect("the sickness armed");
        // The port runs the quick action BEFORE the tick's `UpdateBuffs`
        // head (the quick-mount convention), so the first decrement lands
        // the same tick: 3600 - 1. The C# runs UpdateBuffs (`:24535`)
        // before QuickHeal (`:24826`) — the one-tick offset is recorded
        // in ANALYSIS "Hotbar/quick-switch (S1-P-i54)".
        assert_eq!(p.buffs.buff_time[sick], 3599);
        assert_eq!(app.inventory[4].count, 2, "the stack decremented");
        assert_eq!(app.swing, 0.0, "no item animation on the quick drink");
        assert!(
            app.combat_texts.iter().any(|t| t.active),
            "the HealEffect number spawned (:51929-51932)"
        );
        // The held key does not re-fire (the releaseQuickHeal latch,
        // `:24822-24833`) — and the sickness gates it anyway (`:5503`).
        let hp = p.hp;
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.player.as_ref().unwrap().hp, hp);
        assert_eq!(app.inventory[4].count, 2);
    }

    #[test]
    fn s1_p_i54_quick_heal_gates_on_full_life_and_sickness() {
        // `Player.cs:5503` — the gate chain.
        let mut app = switch_app();
        app.inventory[0] = stocked(188, 3);
        app.keys.insert(KeyCode::KeyH);
        // Full life: no drink.
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.inventory[0].count, 3, "full life refuses");
        // Sickness live: no drink even hurt.
        {
            let p = app.player.as_mut().unwrap();
            p.hp = 10;
            p.buffs.add_buff(21, 100);
        }
        app.keys.remove(&KeyCode::KeyH);
        update_player(&mut app, 1.0 / 60.0); // re-arm the release latch
        app.keys.insert(KeyCode::KeyH);
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.inventory[0].count, 3, "the sickness refuses");
        assert_eq!(app.player.as_ref().unwrap().hp, 10);
    }

    #[test]
    fn s1_p_i54_quick_heal_mushroom_uses_the_1800_delay() {
        // `ApplyPotionDelay` (`Player.cs:51875-51880`): the Mushroom (5)
        // arms `mushroomDelayTime` = 1800 (`Item.cs:49971`), not 3600.
        let mut app = switch_app();
        app.inventory[0] = stocked(5, 2);
        {
            let p = app.player.as_mut().unwrap();
            p.hp = 100;
            p.max_hp = 200;
        }
        app.keys.insert(KeyCode::KeyH);
        update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.hp, 115, "the 15 heal");
        let sick = p.buffs.find_buff_index(21).unwrap();
        // 1800 - 1: the same-tick buff decrement (see the 3600 witness).
        assert_eq!(p.buffs.buff_time[sick], 1799, "the mushroom delay");
    }

    #[test]
    fn s1_p_i54_quick_heal_strange_brew_draws_sound_delay_heal_iframe() {
        // RNG order (`UpdatePlayers`): the UseSound's case-2 draws FIRST
        // (`Player.cs:5516`), then `Next(2400, 4201)` (`:51857`), then
        // `Next(healLife, 121)` (`:51895`), then `NextFloat` (`:51898`).
        // The witness mirrors the draws on a CLONE of the pristine stream.
        let mut app = switch_app();
        app.inventory[0] = stocked(3001, 2); // the Strange Brew, healLife 70
        {
            let p = app.player.as_mut().unwrap();
            p.hp = 10;
            p.max_hp = 200;
        }
        let mut expect = app.rngs.stream(RngKey::UpdatePlayers).clone();
        // Mirror `play_use_sound`'s case-2 rolls for the 3001 row's style.
        let style = crate::soundid::SOUND_STYLES
            [crate::item::initial_use_sound(3001).unwrap().0 as usize];
        let mut roll = |lo: i32, hi: i32| expect.next_range(lo, hi);
        let _ = crate::sound::item_use_case2(style.style, &mut roll);
        let expected_delay = expect.next_range(2400, 4201);
        let expected_heal = expect.next_range(70, 121);
        let expected_iframe_roll = expect.next_float();
        // The HealEffect's two position-jitter draws (`CombatText.cs:88-89`)
        // follow — the witness does not assert them, but they must stay
        // AFTER the heal rolls (the spawn runs last in `quick_heal`).
        let _ = &mut expect; // the twin stream stays aligned for a future pin
        app.keys.insert(KeyCode::KeyH);
        update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        let sick = p.buffs.find_buff_index(21).unwrap();
        // -1: the same-tick buff decrement (see the 3600 witness).
        assert_eq!(p.buffs.buff_time[sick], expected_delay - 1, "the delay roll");
        assert_eq!(p.hp, 10 + expected_heal, "the heal roll");
        let expected_immune = if expected_iframe_roll <= 0.1 {
            240
        } else if expected_iframe_roll <= 0.3 {
            120
        } else if expected_iframe_roll <= 0.6 {
            60
        } else {
            0
        };
        if expected_immune > 0 {
            // -1: the same-tick `update_immunity` decrement (the port's
            // quick action precedes the tick head — see the 3600 witness).
            assert_eq!(p.immune_time, expected_immune - 1, "the i-frame ladder");
        }
    }

    #[test]
    fn s1_p_i54_quick_mount_plays_the_use_sound_without_a_swing() {
        // `Player.cs:6086-6093` — the UseSound at base.Center after
        // SetMount; `Mount.cs:6196-6305` never animates. The port's old
        // swing/attack_cd arming was an invention (S1-P-i54 removed it).
        let mut app = switch_app();
        app.inventory[0] = stocked(2430, 1); // the Slimy Saddle
        app.keys.insert(KeyCode::KeyR);
        let before = app.audio.played_count();
        update_player(&mut app, 1.0 / 60.0);
        assert!(app.player.as_ref().unwrap().mount.is_active());
        assert_eq!(app.audio.played_count(), before + 1, "the saddle chime");
        assert_eq!(app.swing, 0.0, "no swing on the quick mount");
        assert_eq!(app.attack_cd, 0.0, "no attack cooldown either");
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

    // ---- S1-P-i37: the QuickMana key edge (WRITTEN NOT RUN) ----------------

    /// S1-P-i37 — `Player.cs:24834-24845`: the `controlQuickMana` press edge
    /// fires `QuickMana` once (the `releaseQuickMana` latch); holding the
    /// key does not re-drink. The QuickHeal twin (`:24822-24833`) is the
    /// i54 hotbar worker's seam — not duplicated here.
    #[test]
    fn s1_p_i37_the_quick_mana_key_drinks_once_per_press() {
        let mut app = ammo_app();
        app.inventory[7] = crate::item::ItemSlot {
            kind: 110, // the Lesser Mana Potion (healMana 50)
            name: "Lesser Mana Potion",
            count: 5,
            prefix: 0,
        };
        {
            let p = app.player.as_mut().unwrap();
            p.mana = 0;
            p.mana_max = 200;
            // S1-P-i49's per-tick rebase (`Player.cs:18389` statManaMax2 =
            // statManaMax): the PERMANENT cap must carry the fixture's 200
            // too, or ResetEffects rebases to the spawn default 20 and the
            // second drink clamps away (gate V4 witness fix).
            p.mana_max_base = 200;
        }
        // The silent listener: the use-sound draws nothing, the HealMana
        // text's two jitter draws are the only stream spend.
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app.keys.insert(KeyCode::KeyJ); // the default cMana bind
        tick_ammo(&mut app);
        // The press drank: 50 restored, plus at most a few regen points the
        // same tick's UpdateManaRegen may add (the rig's bar started empty).
        let mana_after = app.player.as_ref().unwrap().mana;
        assert!(
            (50..=60).contains(&mana_after),
            "the press drank 50 (+regen), got {mana_after}"
        );
        assert_eq!(app.inventory[7].count, 4);
        assert!(app
            .player
            .as_ref()
            .unwrap()
            .buffs
            .find_buff_index(94)
            .is_some());
        // Held: the latch dropped, no second drink.
        for _ in 0..30 {
            tick_ammo(&mut app);
        }
        assert_eq!(app.inventory[7].count, 4, "one drink per press edge");
        // Release + re-press: the latch re-arms, the second drink lands.
        app.keys.remove(&KeyCode::KeyJ);
        tick_ammo(&mut app);
        let mana_before = app.player.as_ref().unwrap().mana;
        app.keys.insert(KeyCode::KeyJ);
        tick_ammo(&mut app);
        assert_eq!(app.inventory[7].count, 3, "the re-press drank again");
        let mana_now = app.player.as_ref().unwrap().mana;
        assert!(
            mana_now >= mana_before + 50,
            "the second drink restored 50: {mana_before} -> {mana_now}"
        );
    }
}
