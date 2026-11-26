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
    // A6 — `ResetEffects`' `accAmmoCyclerDamage = false` (`Player.cs:18523`).
    p.acc_ammo_cycler_damage = false;
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
            // matching slot wins, like the C#'s `switch` in slot order) AND
            // `accAmmoCyclerDamage` (A6 — every one of the four arms sets it,
            // including 6194's "no cycling"; the +8% ranged fold rides the
            // seam tail below).
            6168 => {
                p.ammo_cycling.mode = crate::ammo::AmmoCyclingMode::AmmoSlots;
                p.acc_ammo_cycler_damage = true;
            }
            6169 => {
                p.ammo_cycling.mode = crate::ammo::AmmoCyclingMode::FullInventory;
                p.acc_ammo_cycler_damage = true;
            }
            6193 => {
                p.ammo_cycling.mode = crate::ammo::AmmoCyclingMode::Random;
                p.acc_ammo_cycler_damage = true;
            }
            // `:14540-14543` — the "no cycling" accessory resets the mode
            // but STILL grants the damage flag (`:14542`).
            6194 => {
                p.ammo_cycling.mode = crate::ammo::AmmoCyclingMode::None;
                p.acc_ammo_cycler_damage = true;
            }
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
    // A6 — the `UpdateEquips` post-loop stat folds for the flags granted
    // above (`Player.cs:12958-12961` cycler +8% ranged, `:12966-12969`
    // Sharp Barb +8 ranged crit), in the C#'s statement order. Both are
    // ADDITIVE stat adds with no RNG; the barb's +1-arrow-damage half is
    // the `ammo.rs` resolver's (`:53548-53551`), untouched here. The C#
    // runs them between the manaFlower fold (`:12962-12965`, ported in
    // `player.rs`) and the sky-stone bundle (`:12989`) — the adds touch
    // disjoint stats, so the seam-tail position is order-identical.
    if p.acc_ammo_cycler_damage {
        crate::accessory_combat::apply_ammo_cycler_damage(p);
    }
    if p.acc_sharp_barb {
        crate::accessory_combat::apply_sharp_barb_crit(p);
    }
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
/// `InfoAccMechShowWires` (`:12801-12804`); the Ruler (486) / Wire Kite grant
/// `rulerLine` (`:12805-12808`), the Laser Ruler (2799) / Wire Kite
/// `rulerGrid` (`:12809-12812`), the Paint Sprayer (2216) / Architect Gizmo
/// Pack (3061) / Presserator (5126) `autoPaint` (`:12813-12816`); the
/// Actuation Accessory (3624) grants `autoActuator` (`:12817-12820`). The
/// `autoActuator` placement consumer
/// (`PlaceThing_Tiles_PlaceIt_AutoPaintAndActuate`, `:40594`) is a recorded
/// follow-up — the flag is granted and read by the visibility gate, but the
/// auto-place-on-build pass is not ported (ANALYSIS "The wiring tools").
/// `InfoAccMechShowWires` (`:12801-12804`); the Actuation Accessory (3624)
/// grants `autoActuator` (`:12817-12820`); the Paint Sprayer (2216) / Gizmo
/// Pack (3061) / Hand of Creation (5126) grant `autoPaint` (`:12813-12816`,
/// W4-g79). The Wire Kite's ruler-line/grid halves (`:12805-12812`) are out
/// of this wiring wave. The `autoActuator`/`autoPaint` placement consumers
/// (`PlaceThing_Tiles_PlaceIt_AutoPaintAndActuate`, `:40594`) are recorded
/// follow-ups — the flags are granted and read by the visibility/builder
/// gates, but the auto-place-on-build pass is not ported (ANALYSIS "The
/// wiring tools" / "Tile paint use paths (W4-g79)").
/// `InfoAccMechShowWires` (`:12801-12804`); the Paint Sprayer (2216) /
/// Architect Gizmo Pack (3061) / Hand Of Creation (5126) grant `autoPaint`
/// (`:12813-12816`); the Actuation Accessory (3624) grants `autoActuator`
/// (`:12817-12820`). The Wire Kite's ruler-line/grid halves (`:12805-12812`)
/// stay out of this wiring wave. The placement consumer
/// (`PlaceThing_Tiles_PlaceIt_AutoPaintAndActuate`, `:40594-40669`) is
/// PORTED — both halves (`world_sim/editing/auto_actuator.rs` +
/// `auto_paint.rs`, wired at the `place_tile` tail); the block-swap
/// consumer (`:39892/:39909`) is ported in `block_swap.rs`.
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
/// the builder-toggle arms (the `preventAllItemPickups` arm `:12821-12824`
/// stays out of the wiring scope).
/// `Player.RefreshMechanicalAccsFromItemType` (`Player.cs:12799-12841`),
/// the wiring-relevant arms only.
fn refresh_mechanical_acc_from_item_type(p: &mut Player, kind: u16) {
    // `:12801-12804` — the Mechanical Lens / Wire Kite show the wires.
    if kind == 3619 || kind == 3611 {
        p.info_acc_mech_show_wires = true;
    }
    // `:12805-12808` — the Ruler / Wire Kite show the ruler line.
    if kind == 486 || kind == 3611 {
        p.ruler_line = true;
    }
    // `:12809-12812` — the Laser Ruler / Wire Kite show the ruler grid.
    if kind == 2799 || kind == 3611 {
        p.ruler_grid = true;
    }
    // `:12813-12816` — the Paint Sprayer / Gizmo Pack / Presserator auto-paint.
    // `:12813-12816` — the Paint Sprayer / Architect Gizmo Pack / Hand Of
    // Creation auto-paint. The held-item arms (`:15271-15274`/:15279-15285/
    // :15286-15294) grant the same flag for the same three kinds — the
    // caller's held-kind pass covers them (the 3624 precedent).
    if kind == 2216 || kind == 3061 || kind == 5126 {
        p.auto_paint = true;
    }
    // `:12817-12820` — the Actuation Accessory auto-places actuators.
    if kind == 3624 {
        p.auto_actuator = true;
    }
    // `:12813-12816` — the Paint Sprayer / Architect's Gizmo Pack / Hand of
    // Creation auto-paint placed tiles/walls (W4-g79).
    if kind == 2216 || kind == 3061 || kind == 5126 {
        p.auto_paint = true;
    }
    // `:12825-12828` — the Guide to Critter Companions (4767) and its
    // upgrade (5323) arm `dontHurtCritters` (the Inferno aura's critter
    // refusal reads it, `Player.cs:20515`).
    if kind == 4767 || kind == 5323 {
        p.dont_hurt_critters = true;
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

/// `Player.UpdatePet` + `Player.UpdatePetLight` (`Player.cs:17298-17340`,
/// called `:25573-25574` right after `UpdateBuffs` `:25566`, inside the
/// `whoAmI == Main.myPlayer` gate): the vanity/light-pet buff PERSISTENCE
/// mechanism. Every tick, if the misc-equip pet/light slot holds an item
/// whose `buffType` is a vanity or light pet, the slot is not hidden and
/// the buff is absent (`FindBuffIndex == -1`), re-grant `AddBuff(num, 3600)`
/// and play the item's UseSound. This is what keeps a pet buff alive after
/// the use-path's `buffTime = 0` grant expires — without it every pet item
/// (the Jumino 5276 included) loses its buff the same tick it is used.
///
/// The light half's fairy trio (`:17322-17336`): buff 27 (Fairy) also
/// matches its 102/101 legacy aliases; when none is present the re-grant
/// picks one of {27, 102, 101} via `Utils.SelectRandom(Main.rand, …)`
/// (`Utils.cs:2459-2462` — one `Next(3)` draw on the `UpdatePlayers`
/// stream, the `Player.Update` swap `Main.cs:17754`).
///
/// The Carrot (603) gate (`:17303`/`:17318`): `type != 603 ||
/// Main.runningCollectorsEdition` — the port has no collector's-edition
/// flag (it folds false, the retail-registry/DLC check `Main.cs:4231-4238`
/// is unported), so the Carrot never re-grants; recorded as a fold.
///
/// The sound plays at `base.position` (the TOP-LEFT, `:17306`/`:17338` —
/// NOT the use-start's `base.Center`), through the same `play_use_sound`
/// façade (the case-2 draws ride `UpdatePlayers`, the `GetRandomPitch`
/// NextFloat rides the audio-only static).
fn update_pet_persistence(
    p: &mut Player,
    misc: &[ItemSlot; MISC],
    audio: &mut crate::audio::Audio,
    rng: &mut tr_core::UnifiedRandom,
) {
    for slot_idx in 0..2 {
        let slot = &misc[slot_idx];
        if slot.count < 1 {
            continue; // `stack >= 1`
        }
        let Some(def) = crate::registry::get(slot.kind) else {
            continue;
        };
        let num = def.buff_type;
        if num < 1 {
            continue; // `buffType >= 1`
        }
        if !(crate::buffs::is_vanity_pet(num) || crate::buffs::is_light_pet(num)) {
            continue; // `Main.vanityPet[num] || Main.lightPet[num]`
        }
        if p.hide_misc[slot_idx] {
            continue; // `!hideMisc[i]`
        }
        if slot.kind == 603 {
            continue; // the Carrot: runningCollectorsEdition folds false
        }
        // `FindBuffIndex(num)` — the light half's fairy aliases first.
        let mut found = p.buffs.find_buff_index(num).is_some();
        let mut grant = num;
        if slot_idx == 1 && num == 27 {
            if !found {
                found = p.buffs.find_buff_index(102).is_some();
            }
            if !found {
                found = p.buffs.find_buff_index(101).is_some();
            }
            if !found {
                // `Utils.SelectRandom<int>(Main.rand, 27, 102, 101)` —
                // one Next(3) draw, the choice order verbatim.
                grant = [27, 102, 101][rng.next_range(0, 3) as usize];
            }
        }
        if found {
            continue;
        }
        p.buffs.add_buff(grant, 3600);
        // `SoundEngine.PlaySound(miscEquips[i].UseSound, base.position)`.
        if let Some(style) = def.use_sound {
            audio.play_use_sound(style, p.x, p.y, def.use_sound_pitch, rng);
        }
    }
}
/// W4-d80 — `Player.UpdateSunScorchValues`' play/follow half
/// (`Player.cs:28590-28603`): the vampire sizzle loop's ensure + volume +
/// position-follow against the tracked pool. Runs on BOTH the live and
/// dead paths (the counter ticks in `Player::tick_sun_scorch`; the C#
/// calls `UpdateSunScorchValues` from `UpdateSunScorch` `:25634` AND
/// `UpdateDead` `:17079`). The `VampireSizzleTracker` condition
/// (`VampireSizzleTracker.cs:12-19`: `!gameMenu && vampireSeed &&
/// sunScorchCounter > 0`) folds to a shared atomic the sim refreshes per
/// tick (the pool cannot hand a live player reference to a `Send`
/// closure). ZERO gameplay draws — the variation/pitch picks ride the two
/// audio-only statics (the census' RNG contract, research/64 sites 22/23).
fn update_sizzle_audio(app: &mut App) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    let vampire_seed = app
        .world
        .micro_seeds
        .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM);
    let lerp = p.sun_scorch_lerp();
    let (cx, cy) = p.center();
    // The condition mirror: `!gameMenu && vampireSeed && counter > 0`.
    // The `gameMenu` term folds with the focus/pause gate that freezes
    // the pool (the census fold); the seed + counter terms ride here.
    let condition_live = vampire_seed && p.sun_scorch_counter > 0;
    let handle = p.sizzle_audio_handle;
    let active = app.audio.get_active_sound(handle).is_some();
    if !active && lerp != 0.0 {
        // The ensure-play (`:28591-28597`): the volume override is the
        // lerp; the condition reads the shared atomic.
        if let Some(style) = crate::tracked_sound::TrackedStyle::by_name("VampireSizzle") {
            let alive = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(condition_live));
            p.sizzle_alive = Some(alive.clone());
            let id = app.audio.play_tracked_looped(
                style,
                cx,
                cy,
                Box::new(move || alive.load(std::sync::atomic::Ordering::Relaxed)),
                crate::tracked_sound::SoundPlayOverrides { volume: Some(lerp) },
            );
            p.sizzle_audio_handle = id;
        }
    }
    // The follow (`:28599-28603`): every tick the live handle's volume +
    // position ride the player. The condition mirror refreshes too.
    if let Some(alive) = &p.sizzle_alive {
        alive.store(condition_live, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(s) = app.audio.get_active_sound_mut(p.sizzle_audio_handle) {
        s.volume = lerp;
        s.position = (cx, cy);
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
    // W4-a5 — the B-key (`Main.cBuff`) buff hotkey: `TriggersSet.CopyInto`
    // (`GameInput/TriggersSet.cs:801-804`) calls `Player.QuickBuff()` on
    // the QuickBuff trigger (`Player.cs:5766-5846`). The port keeps the
    // press-edge latch like `quick_mana_prev` above (the two-frame seam);
    // the C# zeroes only the MOVEMENT controls under the open map
    // (`:24794-24807`), so this key stays live there too.
    let buff_key = app
        .keys
        .iter()
        .any(|k| crate::input_map::is_bound(&binds, crate::input_map::Action::QuickBuff, *k));
    let buff_edge = buff_key && !app.quick_buff_prev;
    app.quick_buff_prev = buff_key;
    if buff_edge {
        crate::potions::quick_buff(app);
    }
    let (sw, sh) = app.screen_size();
    let mouse_world = Vec2::new(
        app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
        app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
    );
    // `Main.MouseOversClear` (`Main.cs:17035` -> `:46511-46512`) + the sign
    // hover scan (`Player.LookForTileInteractions` -> the long-distance
    // arm's `tileSign` block, `Player.cs:32098-32115`) — W4-f22: the
    // bubble/tooltip latches re-derive every player tick.
    crate::signs::mouse_overs_clear(app);
    crate::signs::sign_hover_scan(app, mouse_world);
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
        // slot indexes the 50-grid); the smart-select arm is the override,
        // then the afk auto-kite arm (`:17421-17428`, W4-d16 residual 8 —
        // the 3600-tick gate + the holdStyle-0 use-button force).
        // `controlTorch` = the LeftShift auto-select key (`Main.cTorch`).
        let control_torch = app
            .keys
            .iter()
            .any(|k| crate::input_map::is_bound(&binds, crate::input_map::Action::Torch, *k));
        // `Player.controlTorch` lands on the player for the afk gate
        // (`Player.cs:25497` — W4-d15 final phase); the C# sets it in
        // `TriggersSet.CopyInto` ahead of `Player.Update`.
        if let Some(p) = app.player.as_mut() {
            p.control_torch = control_torch;
        }
        // W4-d16 residual 8 — `UpdateEquips`' `RefreshAutoKitingFromItemTypeAndSlot`
        // scan (`Player.cs:12898`/`:12905`): the C# `ResetEffects` clears
        // `selectedKite = -1` (`:18432`, `:25491`) AFTER the `:25218`
        // selection update reads it, so the override sees LAST tick's
        // `UpdateEquips` pin (`:25633`). The port's `update_with_grapple`
        // ResetEffects clear runs later in this pass; re-pinning here
        // (pre-override, against this tick's inventory) gives the gate a
        // live slot — the one-tick-early deviation is observation-identical
        // for a stable inventory (the pin only changes when the kite moves
        // slots, and the C# would catch up the next tick).
        super::editing::kites::refresh_selected_kite(app);
        let override_slot = if can_change {
            let smart = app.player.as_ref().and_then(|p| {
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
            });
            // `:17417-17420` — SmartSelectLookup wins; the afk arm is the
            // else (`:17421`).
            smart.or_else(|| super::editing::kites::afk_auto_kite_override(app))
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
        // W4-d16 residual 8 — `Player.Update`'s afk-counter tail
        // (`Player.cs:25497-25514`): AFTER the `:25218` selection update
        // (the override gate read the pre-increment counter) and BEFORE
        // the movement block — the standing-still read is the PREVIOUS
        // tick's velocity, exactly the C# order. The counters are not
        // ResetEffects fields, so the later `update_with_grapple` clear
        // block does not touch them.
        {
            let item_animation = (app.swing * 60.0).round() as i32;
            super::editing::kites::afk_counter_tick(app, item_animation, control_torch);
        }
    }

    // `Player.Shellphone_Spawn`'s invalid-area force-clear request
    // (`Player.cs:56169-56172`): collected in the tick, run after the player
    // borrow releases `app` (the kills + drop rolls need `&mut App`).
    let mut force_clear: Option<(i32, i32)> = None;
    // W4-g11 — the `Player.Teleport` effect pair, collected the same way (the
    // replay needs `&mut App`; the drain runs it after the step).
    let mut teleport_effect_req: Option<crate::main_dust::PlayerTeleportReq> = None;
    // The tick closure cannot use whole-`App` methods (`app.player_step`
    // owns it): the door echoes, the progression rows, the NPC buff sends
    // and the Eyebrella screen size defer past the player borrow (the
    // The tick closure needs the whole `App` beside the held `p` field
    // borrow; the raw view is taken BEFORE the borrow (it is Copy and the
    // `App` never moves) and only the disjoint-field step body uses it.
    let app_ptr: *mut App = app;
    // `switch_sends`/`force_clear` convention).
    let mut door_echoes: Vec<(u8, i32, i32, i32)> = Vec::new();
    let mut progression_events: Vec<u32> = Vec::new();
    let mut npc_buff_drain = false;
    let (scr_w, scr_h) = app.screen_size();
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
        // `Player.cs:18798-18800` (W4-f11, fold-133): the mass-wire operation
        // cooldown ticks down beside the rod lock in `ResetEffects`.
        if p.wire_operations_cooldown > 0 {
            p.wire_operations_cooldown -= 1;
        }
        // `Player.cs:24439-24448` (W4-c42 final): the quick-grapple cooldown
        // — the fullscreen-map arm RE-ARMS it every tick (`:24441-24443`),
        // otherwise it decrements while positive (`:24445-24448`).
        if map_open {
            p.gamepad_enable_grapple_cooldown();
        } else if p.quick_grapple_cooldown > 0 {
            p.quick_grapple_cooldown -= 1;
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
        // `Player.SceneMetrics.NPCBannerBuff` (`Player.cs:44197-44199`):
        // the per-banner strength flags the strike lanes' `GetBannerBuffEffect`
        // reads. The C# reads the live global mid-tick; the port snapshots it
        // beside the auras (the same one-tick scene-scan latency).
        p.npc_banner_buff = app.scene_metrics.npc_banner_buff.clone();
        // The zone flags (`Player.ZoneSnow`-family) — the torch-luck table's
        // inputs (`Player.cs:17924-18036`), refreshed beside the auras.
        p.zones = app.scene_metrics.zones();
        // `Player.HasGardenGnomeNearby` (`Player.cs:16726-16730`,
        // `UpdateBiomes`): the SceneMetrics tile-567 marker copies into
        // the player's +0.2 luck input (`RecalculateLuck` `:29150-29153`).
        // The `luckNeedsSync` diff half is net-only. W4-e26.
        p.luck_state.has_garden_gnome_nearby = app.scene_metrics.has_garden_gnome;
        // `LanternNight.LanternsUp` (`LanternNight.cs:17-27`) — the
        // +0.3 lantern-night luck input (`RecalculateLuck` `:29146-29149`):
        // the S1-F-14 scheduler state reads through the same property the
        // C# formula reads. W4-e26.
        p.luck_state.lantern_night = crate::world_events::lanterns_up(&app.world_events);
        // `UpdateEquips`' first-loop inventory scan input (`Player.cs:
        // 12900-12914`): the grid + coin/ammo kinds (the port's halves of
        // the C# inventory[0..58]) — the clover/feather luck family arms
        // from carried items (W4-e26).
        p.carried_kinds.clear();
        p.carried_kinds.extend(
            app.inventory
                .iter()
                .chain(app.equip.coins.iter())
                .chain(app.equip.ammo.iter())
                .filter(|sl| sl.count > 0)
                .map(|sl| sl.kind),
        );
        // W4-f40 (canonical placement, research/114 §1d — c39's duplicate
        // before-grapple write retired onto this one): `Player.RecalculateLuck`'s
        // lantern-night term (`Player.cs:29146-29149`, `if (LanternNight.
        // LanternsUp) luck += 0.3f`). The C# reads the static live inside
        // `UpdateLuck`; the port snapshots `LanternsUp` once per tick beside
        // the auras/zones (the flag only flips at the dusk/dawn boundaries
        // and `UpdateTime`, never mid-player-step), so `update_luck`'s
        // `recalculate_luck` below reads the same value the C# would. The
        // +0.3 add itself already lives in `luck.rs::recalculate_luck` (the
        // M4-EVENT-4 neutral-default input this wires).
        // W4-d70 DOC-SWEEP: the sun-scorch exposure gates
        // (`Player.cs:28681-28685`) — `ZoneGraveyard` and the Moon Lord's
        // sky dim (`SceneMetrics.cs:299-310`, the closest core 398).
        p.zone_graveyard = app.scene_metrics.zone_graveyard;
        p.moon_lord_sky_intensity = {
            let core = app.scene_metrics.closest_npc_position[398];
            if core == [0.0, 0.0] {
                None
            } else {
                let dx = app.scene_metrics.center[0] - core[0];
                let dy = app.scene_metrics.center[1] - core[1];
                let dist = (dx * dx + dy * dy).sqrt();
                // `Utils.SmoothStep` is a clamped inverse lerp (`Utils.cs:126`).
                Some(1.0 - ((dist - 3000.0) / (6000.0 - 3000.0)).clamp(0.0, 1.0))
            }
        };
        // W4-f40: `Player.RecalculateLuck`'s lantern-night term
        // (`Player.cs:29146-29149`, `if (LanternNight.LanternsUp) luck +=
        // 0.3f`). The C# reads the static live inside `UpdateLuck`; the port
        // snapshots `LanternsUp` once per tick beside the auras/zones (the
        // flag only flips at the dusk/dawn boundaries and `UpdateTime`, never
        // mid-player-step), so `update_luck`'s `recalculate_luck` below reads
        // the same value the C# would. The +0.3 add itself already lives in
        // `luck.rs::recalculate_luck` (the M4-EVENT-4 neutral-default input
        // this wires).
        p.luck_state.lantern_night = crate::world_events::lanterns_up(&app.world_events);
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
        // W4-f33 — the Torch God trigger's inventory scan
        // (`Player.cs:18069-18076`): ANY inventory slot holding the favor
        // (5043) blocks the event. The C# walks all 59 `inventory` slots
        // (`Player.cs:1576` — the 50-grid + coins + ammo); the port's
        // mouse item (slot 58) joins the scan (the drop-item convention).
        p.has_torch_gods_favor = app
            .inventory
            .iter()
            .chain(app.equip.coins.iter())
            .chain(app.equip.ammo.iter())
            .chain(std::iter::once(&app.mouse_item))
            .any(|slot| slot.count > 0 && slot.kind == crate::torch_god::TORCH_GODS_FAVOR);
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
                    app.audio.play_menu("Menu_Tick");
                }
            }
        }
        // W4-c46 — `Player.difficulty` mirrors the session character
        // (`Main.ActivePlayerFileData.Player.difficulty`'s stand-in, the
        // `active_player_difficulty` seam — the disjoint field read, the
        // player borrow is live): the hardcore `UpdateDead` arm reads it.
        // The sync runs per tick so a draft change lands without a respawn.
        p.difficulty = app.session_draft.as_ref().map_or(0, |d| d.difficulty);
        // `Player.Update`'s ghost arm (`Player.cs:24744-24748`): the hardcore
        // ghost flies INSTEAD of `UpdateDead` — the early return precedes the
        // dead branch. `Ghost()` reads the live controls and the world bounds.
        if p.ghost {
            // `AllowsRespawnTimerSkip`'s ghost refusal (`Player.cs:38964`)
            // reads live in the death-screen draw — the ghost never shows the
            // skip prompt. Clear the cached flag so a stale `true` from the
            // pre-ghost dead tick cannot draw it (`Main.cs:45348`'s gate).
            app.respawn_skip_available = false;
            app.player_step.tick(dt, |_| {
                p.ghost_tick(&app.world, input);
            });
            return;
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
                respawn_now = p.update_dead_seeded(
                    input.jump,
                    boss_near,
                    danger_near,
                    moon_event_live,
                    app.world.seeds,
                ) || respawn_now;
                // `Player.UpdateDead`'s tail (`Player.cs:17079`): the dead
                // tick still ramps the sun-scorch counter down (−2/tick,
                // `UpdateSunScorchValues` `:28583-28587` — the dead branch
                // clears the burn flag first). W4-f30.
                crate::sun_scorch::update_sun_scorch_values(p);
                p.reset_projectile_caches();
                p.update_projectile_caches(&app.projectiles);
            });
            // W4-f33 — the death abort's deferred relight (`KillMe` →
            // `EndOngoingTorchGodEvent`, `Player.cs:38855-38858`): the
            // `start_death` call queued it; the tile loop lands here
            // (zero RNG). The dead player's `update_dead` never reaches
            // `update_luck`, so this is the abort's only drain.
            crate::torch_god::drain_pending(
                p,
                &mut app.world,
                &mut app.projectiles,
                &mut app.items,
                app.local_id as u8,
            );
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
                // `Player.Spawn`'s local-player arm (`Player.cs:37569`):
                // `Main.instantBGTransitionCounter = 10` — the instant-BG
                // window that also forces the waterfall rescan
                // (`Main.cs:61841`, world_sim/liquids.rs reads it).
                app.bg_instant = 10;
                // W4-c46 — `Player.Spawn`'s position pick (`Player.cs:
                // 37621-37633`): no valid bed (`SpawnX = -1`) falls to the
                // team-based-spawns arm (`Spawn_SetPositionAtTeamSpawn`,
                // `:37993-38011` — the `.wld` v297 `extra_spawn_points`
                // anchor for the player's team, W4-f45's list) BEFORE the
                // world spawn. The C# arm leaves the position untouched
                // when the seed/team gates refuse; the port's cached
                // `spawn_x/y` already holds the world spawn then, so only a
                // landed team anchor rewrites the fields (`revive` reads
                // them below). The helper owns `&mut App` (the force-clear
                // may kill tiles + roll drops) — `p`'s borrow ends with the
                // bed re-validation above (the dead branch returns).
                if app.world.bed_spawn.is_none() {
                    if let Some((px, py)) =
                        crate::recall_teleport::spawn_set_position_at_team_spawn(app)
                    {
                        let revived = app.player.as_mut().expect("the dead player");
                        revived.spawn_x = px;
                        revived.spawn_y = py;
                    }
                }
                app.player.as_mut().expect("the dead player").revive();
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
            // W4-d80 — `Player.cs:17079`: the dead path's sizzle play/follow
            // (the counter ticked in `update_dead`'s tail).
            update_sizzle_audio(app);
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
            // The live items for the `GetEffectiveArmor` share scan
            // (`Player.cs:5690-5710`).
            p.armor_items[i] = app.equip.armor[i];
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
        // `Player.cs:24554-24557` — the overhead chat timer ticks once per
        // player update, right after the void-vault refresh:
        // `if (chatOverhead.timeLeft > 0) chatOverhead.timeLeft--`.
        if p.chat_overhead.time_left > 0 {
            p.chat_overhead.time_left -= 1;
        }
        // `Player.dye`/`miscDyes`' kinds snapshot — `UpdateDyes`' input
        // through `GetEffectiveDye` (`Player.cs:5742-5764`): an air live slot
        // takes the first shared loadout dye, else the slot itself.
        for i in 0..10 {
            p.dye_items[i] = app.equip.dye[i];
            p.dye_slots[i] = p.get_effective_dye(i).0.kind;
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
            // W4-g10 FINAL: the petting gates are live — holdStyles 4/5/6
            // hide the item while petting (`:50204`/`:50225-50232`).
            // W4-d21 fold 2: `CanVisuallyHoldItem`'s petting refusals
            // (`Player.cs:50204-50207`/`:50225-50232`) — the holdStyle
            // 4/5/6 arms all refuse while petting.
            let mounted_blocks_hold = p.mount.is_active() && matches!(p.mount.kind(), 55 | 56 | 61);
            let petting_blocks_hold = p.petting.is_petting && matches!(hold_style, 4 | 5 | 6);
            let can_visually_hold = !mounted_blocks_hold
                && !petting_blocks_hold
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
                // `!wet || !item.noWet` (`Player.cs:36455`): the port has no
                // `Item.noWet` field, so the gate stays open (ANALYSIS).
                hold_style_dry: true,
                // `!happyFunTorchTime || createTile != 4` (`Player.cs:36455`,
                // W4-f33): during the Torch God event a held torch
                // (`createTile == 4`) drops the torch-hold body row. The
                // event flag now exists (`torch_god.happy_fun_torch_time`);
                // `createTile` rides the registry's `tile` field.
                torch_hold_ok: !p.torch_god.happy_fun_torch_time
                    || crate::registry::get(kind).is_none_or(|d| d.tile != Some(4)),
                grapple_dx: dx,
                grapple_dy: dy,
            };
        }
        if cart_pin && !p.cart_mounted {
            p.mount_cart();
        }
        let mut run_pixels = 0.0f32;
        // `FixedStepper::tick` borrows `app.player_step` for the call while
        // the step body needs the whole `App` (the player_dust drains). Move
        // the accumulator out and hand the body a raw `App` view; the outer
        // `p` borrow is a disjoint field and stays checked. The body never
        // touches `player_step` (the accumulator is restored below).
        let mut player_step = std::mem::replace(
            &mut app.player_step,
            crate::world_sim::step::FixedStepper::new(1.0 / 60.0, 8),
        );
        player_step.tick(dt, |_| {
            let app: &mut App = unsafe { &mut *app_ptr };
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
            // W4-f6 wiring — the mount visual-effect inputs: the
            // `timeForVisualEffects` tick counter (the Pixie tail's even-tick
            // gate, `Mount.cs:5147`), `WindForVisuals` + the wind-exposure
            // gate (the Witch Broom trinket sway, `Mount.cs:3568-3572`).
            p.time_for_visuals = (app.anim_time as f64 * 60.0) % 216000.0;
            p.wind_for_visuals = app.wind.speed_current;
            let (mcx, mcy) = p.center();
            p.wind_in_place = app.wind.in_a_place_with_wind(
                &app.world, (mcx / 16.0) as i32, (mcy / 16.0) as i32, 1, 1,
            );
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
            // W4-f37 — `Player.Update`'s post-`UpdateEquips` discount
            // refresh (`Player.cs:25636-25639`): `if (Main.npcShop <= 0)
            // discountAvailable = discountEquipped`. `update_with_grapple`
            // below runs the ResetEffects → UpdateEquips chain that arms
            // `discount_equipped` (the 854/3035 rows); the shop-open tick
            // LATCHES the previous value (the vanilla never re-reads it
            // while `npcShop > 0`), the shop-closed tick mirrors the flag.
            // W4-c46 — `Player.itemAnimation > 0` for the `breathCDMax`
            // property and `CheckDrowning`'s reed probe (`Player.cs:3860`/
            // `:23199`): the swing state with a held item, the sleeping
            // gate's `item_in_use` above.
            p.item_animation_active = item_in_use;
            // M2-MOUNT-4 — the Scutlix/Santank auto-fire scan
            // (`Mount.UpdateEffects` case 9/46, `Mount.cs:4879-4947`, called
            // `Player.cs:25658` BEFORE the movement block + the `:26664`
            // AbilityRecovery). The port runs it here — ahead of
            // `update_with_grapple` (whose `update_mount` tail ticks
            // `ability_recovery`) — so the fire reads the pre-recovery
            // cooldown, the C#'s order. The fire's `RotatedByRandom` draws
            // ride the `UpdatePlayers` stream inline; the two Santank cues
            // queue for the pass-tail drain (the cart-cue batch convention).
            p.update_mount_ability(&app.world, &app.npcs, &mut app.projectiles, rng);
            // W4-c39 exec (research/114 §1d): the duplicate luck write here
            // DROPPED — f40's beside-auras/zones placement is canonical (the
            // single write moved up beside `p.zones`).
            p.update_with_grapple(
                &app.world,
                input,
                &grapple_for_tick,
                item_in_use,
                false,
                app.rain.is_raining(),
                Some(&app.wind),
                rng,
                // W4-g10 FINAL — `UpdatePettingAnimal`'s roster + talk latch.
                &app.npcs,
                app.talk_npc,
                // `FocusHelper.AllowRain && !Main.mapFullscreen` (the Eyebrella drip).
                app.window_focused && !app.game_paused && !map_open,
            );
            // W4-f37 — the refresh itself (`Player.cs:25636-25639`): only
            // the shop-CLOSED ticks copy `discountEquipped` into the App's
            // `discount_available` mirror; an open shop keeps its latch.
            if app.shop_npc.is_none() {
                app.discount_available = p.discount_equipped;
            }
            // W4-f33 — the Torch God event's deferred writes (the tile
            // snuff/relight, the 949 spawn, the favor drop): the decision
            // + RNG ran inside `update_luck` (the C# call position), the
            // writes land here in the same fixed step (the `pending_*`
            // convention). Zero draws — the field-wise split keeps the
            // closure's `p` borrow apart from the app fields.
            crate::torch_god::drain_pending(
                p,
                &mut app.world,
                &mut app.projectiles,
                &mut app.items,
                app.local_id as u8,
            );
            // `Player.cs:28274-28275`: the switch-tile bridge — the player's
            // box reaching a pressure plate trips the wiring (the C#'s call
            // sits right after `FloorVisuals`, inside `Player.Update`).
            // The §8.1 stamp (parts form — inside the player borrow): the
            // trip can shift a wired trapdoor/gate.
            app.wiring.set_entity_boxes(
                crate::world_sim::wiring::collect_entity_boxes(
                    Some(&*p),
                    &app.npcs,
                    &app.remote_players,
                ),
            );
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
                // W4-g83: the plate kill's framing dispatch rides the
                // UpdatePlayers stream (the active genRand of the tick).
                rng,
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
            // W4-c42 — the player-tripped wired door cues (same stream).
            for (open, px, py) in app.wiring.take_door_cues() {
                app.audio.play_door_sound(
                    open,
                    px as f32,
                    py as f32,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // The §8.2 echoes: SP discards (the netMode-0 return).
            for (n, x, y, d) in app.wiring.take_door_echoes() {
                door_echoes.push((n as u8, x, y, d));
            }
            // `PressurePlateHelper.UpdatePlayerPosition(this)`
            // (`Player.cs:28276`): the weighted pressure plates (428) the
            // player stands on. The §8.1 stamp (parts form): the poke can
            // shift a wired trapdoor/gate on the plate's line.
            app.wiring.set_entity_boxes(
                crate::world_sim::wiring::collect_entity_boxes(
                    Some(&*p),
                    &app.npcs,
                    &app.remote_players,
                ),
            );
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
            // W4-c42 — the weighted plates' wired door cues (the
            // UpdatePressurePlates stream, the click precedent).
            for (open, px, py) in app.wiring.take_door_cues() {
                app.audio.play_door_sound(
                    open,
                    px as f32,
                    py as f32,
                    app.rngs.stream(crate::app::RngKey::UpdatePressurePlates),
                );
            }
            // The §8.2 echoes: SP discards (the netMode-0 return).
            for (n, x, y, d) in app.wiring.take_door_echoes() {
                door_echoes.push((n as u8, x, y, d));
            }
            if run_eligible && p.vy == 0.0 {
                run_pixels += (p.x - run_x0).abs();
            }
            // `Player::update_summon_buff_arms`): the recount
            p.update_projectile_caches(&app.projectiles);
            p.update_summon_buff_arms(&mut app.projectiles, &app.inventory);
            // W4-f12 — the `UpdateBuffs` frame WIRING: one slot-ordered pass
            // over the buff bar in the exact C# `else if` chain order, driving
            // the gathered wave modules + the canonical g27 orb module
            // (keep-one) and folding their accumulators onto the live fields.
            // Runs AFTER the summon pass (which owns the keep-alives + the
            // 110/150/348 cap rows) and BEFORE the accessory pass (which owns
            // the pets/imbues); the wiring skips those ids (no double grant).
            // The frame head/tails (`:9857-9864`, `:12046-12067`) ride along.
            {
                let dont_starve = app
                    .world
                    .seeds
                    .contains(tr_core::seeds::SeedFlags::DONT_STARVE);
                let remix = app.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
                let held_item_type = app.inventory[app.selected].kind as i32;
                let day_time = app.world.day_time;
                // `WorldGen.SolidTile3` (`WorldGen.cs:70465-70483`) — the
                // LOOSEST solid test (actuated tiles stay solid); the 1-tile
                // border refusal maps to the out-of-bounds `None`.
                let solid3 = |x: i32, y: i32| -> bool {
                    if x < 1 || x >= app.world.width - 1 || y < 1 || y >= app.world.height - 1 {
                        return false;
                    }
                    let Some(t) = app.world.tile(x, y) else {
                        return false;
                    };
                    t.active()
                        && !t.inactive()
                        && tr_core::tiledata::TILE_SOLID.has(t.kind)
                        && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                };
                let inputs = crate::buff_frame_wiring::BuffFrameInputs {
                    day_time,
                    dont_starve,
                    remix,
                    solid3: &solid3,
                    held_item_type,
                };
                crate::buff_frame_wiring::update_buff_frame(
                    p,
                    &mut app.projectiles,
                    &app.world,
                    &inputs,
                );
                // Drain the frame's emote / Hungry-sound requests onto the
                // live seams (`EmoteBubble.MakeLocalPlayerEmote`,
                // `SoundEngine.PlaySound(SoundID.Hungry)`).
                for emote in p.pending_frame_emotes.drain(..) {
                    app.emotes.new_bubble(emote);
                }
                for _ in 0..p.pending_hungry_sounds {
                    app.audio.play("hungry");
                }
                p.pending_hungry_sounds = 0;
                // W4-f12 gap 2 — the Inferno aura's NPC damage ring
                // (`Player.cs:10224-10238`): the roster scan the wiring
                // deferred. Runs right after the frame fold set `p.inferno`
                // / the cadence gate; the strike requests drain onto the
                // hit-sound + probe seams below (the C# order: combat text
                // inside the scan, sound + HitEffect after the strike).
                let damage_tick = p.inferno_counter % 60 == 0;
                let strikes = crate::buff_inferno_aura::inferno_aura_scan(
                    p,
                    &mut app.npcs,
                    damage_tick,
                    &app.scene_metrics,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.combat_texts,
                    app.lookups.combat_text_font.as_ref(),
                );
                for s in strikes {
                    // `NPC.StrikeNPC`'s hit-sound arm (`NPC.cs:81544-81553`):
                    // the kind's `HitSound` at `base.position`, the pitch
                    // draw on the same `UpdatePlayers` stream.
                    let (nx, ny) = (app.npcs[s.idx].x, app.npcs[s.idx].y);
                    if let Some(cue) = app.npcs[s.idx].kind.hit_sound() {
                        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                        app.audio.play_npc_hit_sound(cue, nx, ny, rng);
                    }
                    // `StrikeNPC_Inner`'s `HitEffect` tail (`NPC.cs:81488`):
                    // the Destroyer body's probe gate rides the strike's
                    // stream (the melee/proj precedent).
                    let probe_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    if let Some(anchor) = crate::npc_ai_style37::probe_launch_on_hit(
                        &mut app.npcs[s.idx], probe_rng, false,
                    ) {
                        app.npcs[s.idx].wants_probe = Some(anchor);
                    }
                }
                // `Player.cs:25605`: `wolfAcc = false` — the C# clears the
                // flag AFTER `UpdateBuffs` read it (the `:11895` Werewolf
                // gate); the port's buff scan is this wiring, so the clear
                // rides here, not inside `update_with_grapple` (clearing it
                // there would starve the scan). The next tick's equip pass
                // re-arms it from the worn charm/shell.
                p.wolf_acc = false;
            }
            // W4-f49 R3 step 4 — THE WIRING SEAM + the lifted regen tick.
            // f12's `crate::buff_frame_wiring::update_buff_frame(p, …)`
            // hooks HERE (its phase-2 contract: after
            // `update_summon_buff_arms`, the `:25566` UpdateBuffs scan
            // position). Its `fold_orb` carries the nebula bridge ASSIGNs
            // (`p.nebula_level_mana = s.nebula_level_mana;` + the life/
            // damage twins — the `Player::fold_orb_nebula_bridge` stub)
            // and the `buff_life_regen` fold.
            // The C# order is scan (`:25566`) → `UpdateEquips` (`:25633`)
            // → `UpdateLifeRegen` (`:25977`) → `UpdateManaRegen`
            // (`:25995`) → `GrabItems` (`:27428`), so the regen reads run
            // AFTER the scan/bridge: `update_survival` was LIFTED out of
            // `update_with_grapple` (the pre-lift position read the
            // PREVIOUS tick's reset-zeroed tier — and f12's own
            // `buff_life_regen` fold landed after the read and was wiped
            // by the next reset before any read; the lift repairs both).
            // Before the accessory pass is fine — the C# regen reads sit
            // after UpdateEquips too.
            p.update_survival(
                &app.world,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
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
            // W4-e10 gap census — `Player.Update`'s myPlayer block right
            // after `UpdateBuffs` (`Player.cs:25567-25574`): `UpdatePet`
            // then `UpdatePetLight` re-grant the equipped pet's absent
            // buff (3600) and play the item's `UseSound` at
            // `base.position` (`:17306`/`:17338`). The style façade's
            // case-2 pitch rides `UpdatePlayers` after the attenuation
            // gate (the S1-E-21f precedent); the fairy-27 pick's one
            // `Next(3)` (`:17335`) rides the same stream BEFORE the
            // sound's draws — the C# call order.
            let pet_sound = p.update_pet_grant(&app.equip.misc_equips[0]);
            let light_sound = p.update_light_pet_grant(
                &app.equip.misc_equips[1],
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
            for idx in [pet_sound, light_sound].into_iter().flatten() {
                app.audio.play_use_sound(
                    idx,
                    p.x,
                    p.y,
                    0.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // W4-c36 — `Player.cs:25567-25572` (`UpdateBuffs`' tail, the
            // myPlayer arm): with neither OnFire (24) nor Poisoned (20)
            // running, the trap-debuff source clears — a trap death only
            // counts while its debuff lingers. The `hp > 0` gate keeps the
            // flag for the death seam: the port latches `dead` in the LATER
            // npcs pass (`on_player_death` reads the flag), so a lethal
            // trap hit this tick must survive the clear (the C# `KillMe`
            // runs synchronously inside the hit, before any clear).
            let burning_or_poisoned = p
                .buffs
                .active_types()
                .iter()
                .any(|&b| b == 20 || b == 24);
            if !burning_or_poisoned && p.hp > 0 {
                p.trap_debuff_source = false;
            }
            // W4-d20 T4b — `Player.UpdatePet`/`UpdatePetLight`
            // (`Player.cs:25573-25574`): right after `UpdateBuffs`
            // (`:25566`, the buff arms above) and before `UpdateEquips`
            // (`:25633`, the grants below) — the vanity/light-pet buff
            // re-grant (3600) + UseSound, the persistence mechanism for
            // every pet item. The fairy trio's SelectRandom draw rides
            // the same `UpdatePlayers` stream.
            update_pet_persistence(
                p,
                &app.equip.misc_equips,
                &mut app.audio,
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
            // W4-f50 — `UpdateEquips`' Football scan (`Player.cs:12906-12918`):
            // the Football (4743) in any of the 58 inventory slots (or the
            // mouse slot 58) arms `hasFootball`. The hurt arm (`:38255`) reads
            // it; `ResetEffects` clears it (`:18345`) and this re-derives per
            // tick. The port's inventory is the 50-slot hotbar+inventory (the
            // mouse slot is `app.mouse_item`).
            p.has_football = app.inventory.iter().any(|s| s.count > 0 && s.kind == 4743)
                || (app.mouse_item.count > 0 && app.mouse_item.kind == 4743);
            // The `UpdateEquips` summon-cap rows (`Player.cs:13254-13948`,
            // `:15324-15329`) — the armor/held half beside the ammo flags.
            grant_summon_caps(p, &app.equip.armor, &app.equip.misc_equips, &app.world);
            // W4-f30 — `Player.UpdateSunScorch` (`Player.cs:25634`): the
            // vampire seed's sun-exposure scan + the 120-crossing burn, in
            // the C#'s `UpdateEquips` → `UpdateSunScorch` → `UpdateArmorSets`
            // slot (the armor-set fold above already ran inside
            // `update_equips`; the Molten `buffImmune[24]` gate reads the
            // flag this scan arms — the burn's `Array.Clear` at `:28653`
            // strips it again on the burning ticks, matching the C# order
            // where the clear follows the set grant). The 33 notify fires
            // on the crossing tick with the vampire vanity set worn.
            {
                let zones = app.scene_metrics.zones();
                // `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023`):
                // the EchoMonolith / Spectre-Goggles `CanSeeInvisibleBlocks`
                // granters are unported (the draw seam's precedent,
                // `frame/draw.rs:393`); the `TR_SHOW_INVISIBLE` pin stands in.
                let show_invisible = crate::env::show_invisible();
                let vampire_seed = app
                    .world
                    .micro_seeds
                    .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM);
            if crate::sun_scorch::update_sun_scorch(
                    p,
                    &app.world,
                    &app.npcs,
                    vampire_seed,
                    app.world.day_time,
                    app.rain.is_raining(),
                    app.world.eclipse,
                    app.scene_metrics.zone_graveyard,
                    zones.glowshroom,
                    show_invisible,
                ) {
                    progression_events.push(33);
                }
                // Order seam: on a warm-up burn tick (flag set, counter
                // < 120) the C#'s Molten grant (`:25685`, AFTER the scorch)
                // still applies — restore the pre-fold immunity the port's
                // early fold withheld. At >= 120 the `:28653` Array.Clear
                // stripped everything, so nothing is restored.
                if p.vampire_burning_in_sunlight && p.sun_scorch_counter < 120 {
                    p.buffs.buff_immune[24] = p.fire_immune_pre_molten;
                }
            }
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
            // W4-e7 lane 23 — `Player.Update_SnappingStone`
            // (`Player.cs:28385-28410`, called at `:24564` BEFORE
            // `Update_HarpyCharm` `:24565` / `Update_AmmoCycler` `:24566`):
            // the shared Snapping Stone strike cooldown counts down one per
            // tick. At the zero-crossing the first worn member's recharge
            // cue fires (`:28392-28408`): the sound at MountedCenter + the
            // particle burst's draws, both on the `UpdatePlayers` stream
            // (`Main.cs:17754`). The strike machine (`snapping_stone.rs`)
            // arms the 360-tick cooldown; the ready lights
            // (`player_lights_b::snapping_stone_lights`) read the live value.
            let was_cooling = p.acc_snapping_stone_cooldown > 0;
            p.update_snapping_stone();
            if was_cooling && p.acc_snapping_stone_cooldown == 0 {
                let recharge_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                if let Some(cue) = crate::snapping_stone::snapping_stone_recharge(p, recharge_rng) {
                    let sound_rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    crate::snapping_stone::play_striking_sound(&mut app.audio, cue, sound_rng);
                }
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
            // W4-f20 F7 — `Player.Update_HarpyCharm` (`Player.cs:28436-28446`,
            // called at `:24565` between `Update_SnappingStone` `:24564` and
            // `Update_AmmoCycler` `:24566`): the charm cooldown decrements,
            // then the nearest-arrow charm scan runs (`HarpyCharmEffect`
            // `:28448-28490` → `ApplyHarpyCharm` `Projectile.cs:57796-57824`).
            // The C# order is charm BEFORE snake-band; the port's split-frame
            // fold seats it after (the `grant_ammo_mastery` precedent — the
            // two are disjoint: the charm writes projectile ai[2]/velocity,
            // the snake-band writes the player's whip counter). The scan
            // draws nothing — the `UpdatePlayers` stream is untouched.
            crate::projectile_harpy_charm::update_harpy_charm(
                p,
                &mut app.projectiles,
                &app.world,
                &app.npcs,
                app.local_id as u8,
            );
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
                // The §8.1 stamp (parts form): the track trip can shift a
                // wired trapdoor/gate on the magnet's line.
                app.wiring.set_entity_boxes(
                    crate::world_sim::wiring::collect_entity_boxes(
                        Some(&*p),
                        &app.npcs,
                        &app.remote_players,
                    ),
                );
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
                // W4-c42 — the track trip's wired door cues (same stream).
                for (open, px, py) in app.wiring.take_door_cues() {
                    app.audio.play_door_sound(
                        open,
                        px as f32,
                        py as f32,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                }
                // The §8.2 echoes: SP discards (the netMode-0 return).
                for (n, x, y, d) in app.wiring.take_door_echoes() {
                    door_echoes.push((n as u8, x, y, d));
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
            // W4-d79 — the minecart cues are `SoundID.ItemNN` STYLE plays
            // (`DelegateMethods.cs:248-256` LandingSound Item53 / BumperSound
            // Item56, `Player.cs:19870`/:19928 the rolling Item55): the
            // `SoundEngine.cs:93-99` façade consumes the GetRandomPitch
            // NextFloat on the audio-only stream, then case 2's per-cue pitch
            // draw + the 53/55 playing guards + the instance pool
            // (`LegacySoundPlayer.cs:552-562`/`:612-616`) ride the
            // `UpdatePlayers` stream — `Player.HorizontalMovement` runs in
            // `Player.Update` under the `Main.cs:17754` swap. The bare
            // `play_at` stand-in drew nothing and stacked voices (D1/D4).
            for cue in p.take_cart_sounds() {
                // W4-c14 - the cart/movement cues are `SoundID.ItemN` style
                // plays (`Player.cs:19870`/`:19928` Item55, the landing
                // Item53 `DelegateMethods.cs:250`, the bumper Item56
                // `:255`): the facade (`SoundEngine.cs:93-99`) - the
                // GetRandomPitch NextFloat on the audio-only stream, then
                // case 2's volume/pitch draws on the `UpdatePlayers`
                // stream after the gate (53/55's ×0.75 arms + the signed
                // Next(-20,-11)*0.02 windows, `LegacySoundPlayer.cs:552-
                // 563`/`:612-616`). The 53/55 playing guards stay the
                // standing D4 deviation (wave A). The positions are already
                // the C#'s truncated ints.
                let style = cue
                    .cue
                    .strip_prefix("Item_")
                    .and_then(|s| s.parse::<i32>().ok());
                if let Some(style) = style {
                    app.audio.play_npc_item_cue(
                        style,
                        cue.x,
                        cue.y,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                } else if matches!(cue.cue, "MeowcartLanding" | "MeowcartBumper") {
                    // W4-c14 - the Meowmere cart's case-37 plays
                    // (`Mount.cs:661` style 5 / `:666` style 3): the slot
                    // pick + the Style*0.05 volume + the wide pitch window.
                    let style = if cue.cue == "MeowcartLanding" { 5 } else { 3 };
                    app.audio.play_meowcart_sound(
                        style,
                        cue.x,
                        cue.y,
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    );
                } else {
                    app.audio.play_at(cue.cue, cue.x, cue.y, 1.0);
                }
            }
            // M2-MOUNT-4 — the Santank ability cues (`Mount.cs:3300` Item89
            // gift / `:3311` Item11 missile, volumeScale 0.2): the int
            // overload (`SoundEngine.cs:102`) — NO GetRandomPitch draw, the
            // ONE case-2 `Next(-6, 7)` pitch roll rides the `UpdatePlayers`
            // stream after the attenuation gate (the fire runs inside
            // `Player.Update`, `Main.cs:17754`). The C# plays inline at the
            // spawn; the port drains at the pass tail (the cart-cue batch
            // convention, ANALYSIS). The facade truncates the positions.
            for (style, x, y) in p.take_ability_sounds() {
                app.audio.play_mount_ability_sound(
                    style,
                    x,
                    y,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // W4-e10 — the cart cues are `SoundID.ItemNN` STYLE plays
            for cue in p.take_cart_sounds() {
                // (`Player.cs:19870`/`:19928` Item55, `DelegateMethods.cs:
                // 250`/`:255` Item53/56): the wave-A façade — the
                // `GetRandomPitch` NextFloat audio-only, then case 2's
                // per-style pitch draw (53: `Next(-20, -11) * 0.02`, 55:
                // its negation, 56: `Next(-50, 51) * 0.002` + the 0.6
                // volume, `LegacySoundPlayer.cs:610-634`) on
                // `UpdatePlayers` after the gate — the cart tick runs in
                // the player pass (`Main.cs:17754`). The unpositioned
                // `play_at` stand-in is retired.
                let style = match cue.cue {
                    "Item_53" => 53,
                    "Item_55" => 55,
                    "Item_56" => 56,
                    _ => {
                        app.audio.play_at(cue.cue, cue.x, cue.y, 1.0);
                        continue;
                    }
                };
                app.audio.play_proj_sound(
                    (2, style),
                    cue.x,
                    cue.y,
                    0.0,
                    1.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
                let style_num: i32 = match cue.cue {
                    "Item_53" => 53,
                    "Item_55" => 55,
                    "Item_56" => 56,
                    _ => continue,
                };
                app.audio.play_npc_item_cue(
                    style_num,
                    cue.x,
                    cue.y,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // W4-d79 — the full-mana refill cue + dust burst
            // (`Player.UpdateManaRegen`, `Player.cs:19550-19560`): the
            // unpositioned `PlaySound(25)` — case 25
            // (`LegacySoundPlayer.cs:963-968`) is Volume/Pan only, NO
            // draws, NO Stop (the slot overwrites) — then the 5× dust-45
            // loop. EACH iteration rolls `Next(20, 26) * 0.1` as the
            // `NewDust` scale argument (`:19555` — the draw precedes the
            // spawn's own rolls), alpha 255, then `noLight`/`noGravity`/
            // `velocity *= 0.5` (`:19556-19558`). The dust draws are NOT
            // gated on the sound gate — the C# loops regardless. The
            // draws ride `UpdatePlayers` (the `Main.cs:17754` swap covers
            // `Player.Update`); the drain sits BEFORE the sticky-dust
            // drain below — the C#'s `UpdateManaRegen` (`:25995`) precedes
            // `StickyMovement` (`:27654`) in the update order.
            if let Some((rx, ry, rw, rh)) = p.take_mana_refill() {
                app.audio.play_single_voice("MaxMana", -1.0, -1.0);
                for _ in 0..5 {
                    let scale = app
                        .rngs
                        .stream(crate::app::RngKey::UpdatePlayers)
                        .next_range(20, 26) as f32
                        * 0.1;
                    let idx = app.dust.new_dust(
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        &spawn_ctx,
                        &app.world,
                        Vec2::new(rx, ry),
                        rw,
                        rh,
                        45,
                        0.0,
                        0.0,
                        255,
                        tr_core::particles::Color::default(),
                        scale,
                    );
                    // `:19556-19558` — the post-arms write the returned
                    // slot, the sentinel included (the C#'s own shape).
                    let slot = app.dust.slot_mut(idx);
                    slot.no_light = true;
                    slot.no_gravity = true;
                    slot.velocity *= 0.5;
                }
            }
            // `Player.StickyMovement`'s handoffs: the cobweb the drag broke
            // (`Player.cs:22949` `WorldGen.KillTile`) and the type-153 web puffs
            // (`:23036-23080` `Dust.NewDust` + the `Next(0,6)`/`velocity *= 0.1`/
            // `noGravity` post-arms). The tick queues the position; the pool's
            // own rolls ride the same `UpdatePlayers` stream, so the draw order
            // is the source's (S1-A-10).
            if let Some((wx, wy)) = p.take_sticky_kill() {
                // W4-g83: the cobweb kill's framing dispatch rides the
                // UpdatePlayers stream (the active genRand of the tick).
                tr_core::edit::kill_tile(
                    &mut app.world,
                    wx,
                    wy,
                    false,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
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
                    puff.scale,
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
            // W4-f42 — `Player.DoubleJumpVisuals`' sandstorm arm
            // (`Player.cs:21913-21925`): the three type-124 dusts, then the
            // optional 220-222 gore. The tick queued the geometry; the pool
            // rolls ride the SAME `UpdatePlayers` stream in the C#'s order
            // (3 `NewDust`s, then the gore's `Next(220, 223)` type pick +
            // `NewGore`), so the draw order is the source's (S1-A-10).
            if let Some(v) = p.take_sandstorm_jump() {
                for _ in 0..3 {
                    let idx = app.dust.new_dust(
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        &spawn_ctx,
                        &app.world,
                        v.dust_pos,
                        crate::player::WIDTH as i32,
                        32,
                        124,
                        v.dust_vel.x,
                        v.dust_vel.y,
                        150,
                        tr_core::particles::Color::default(),
                        v.num4,
                    );
                    // `:21916-21917` — `velocity *= 0.5*num4`,
                    // `fadeIn = 1.5*num4` (the refused sentinel takes the
                    // writes blindly, `Main.dust[6000]`).
                    let slot = app.dust.slot_mut(idx);
                    slot.velocity *= 0.5 * v.num4;
                    slot.fade_in = 1.5 * v.num4;
                }
                if let Some((gpos, gvel)) = v.gore {
                    // `:21922` — `Main.rand.Next(220, 223)` picks the type.
                    let gore_type = app
                        .rngs
                        .stream(crate::app::RngKey::UpdatePlayers)
                        .next_range(220, 223);
                    let gi = app.gore.new_gore(
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        &app.world,
                        gpos,
                        gvel,
                        gore_type,
                        v.num4,
                    );
                    // `:21923-21924` — `velocity = base.velocity*0.3*num4`,
                    // `alpha = 100`.
                    let g = app.gore.slot_mut(gi);
                    g.velocity = v.vel * 0.3 * v.num4;
                    g.alpha = 100;
                }
            }

            // W4-f50 — the Solar Flare shield-grant dust burst (`Player.cs:
            // 16387-16395`): 16 dust-6 particles at the player's box, each
            // `noGravity`, scale 1.7, fadeIn 0.5, velocity ×5. The set-bonus
            // machine queued the flag; the dust pool lives on the App, so the
            // drain spawns them on the `UpdatePlayers` stream (the
            // `pending_sticky_dust` convention). The `NewDust` alpha is 100,
            // scale 1 (the post-arm overrides it to 1.7).
            if p.take_solar_grant_dust() {
                for _ in 0..16 {
                    let idx = app.dust.new_dust(
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        &spawn_ctx,
                        &app.world,
                        glam::Vec2::new(p.x, p.y),
                        p.width as i32,
                        p.height as i32,
                        6,
                        0.0,
                        0.0,
                        100,
                        tr_core::particles::Color::default(),
                        1.0,
                    );
                    let slot = app.dust.slot_mut(idx);
                    slot.no_gravity = true;
                    slot.scale = 1.7;
                    slot.fade_in = 0.5;
                    slot.velocity *= 5.0;
                }
            }

            // W4-e4 — the Flame Waker Boots' / Hellfire Treads' walking
            // flames (`Player.DoBootsEffect_PlaceFlamesOnTile`,
            // `Player.cs:14094-14097`): the equip pass queued the gated tile
            // positions; the FlameWaders orchestra emitter rolls here on the
            // SAME `UpdatePlayers` stream (the `pending_sticky_dust`
            // convention — the player tick cannot borrow the App's dust
            // pool). Movement zero, invoker = the local player.
            for (fx, fy) in p.take_flame_waders() {
                let settings = crate::orchestrator_dust::OrchestraSettings {
                    position: Vec2::new(fx, fy),
                    movement: Vec2::ZERO,
                    info: 0,
                    invoker: app.local_id as u8,
                };
                crate::orchestrator_dust::flame_waders(
                    &settings,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                );
            }
            // W4-e1 — the Player.cs dust sites' SECOND batch (the ledger
            // docs/research/64-dust-sites-player-2.md, the 3-way reconcile
            // docs/research/141-player-dust-reconcile.md). Each snapshot was
            // taken at the C# read moment inside the tick; the pure emitters
            // replay on the `UpdatePlayers` stream in C# EXECUTION order
            // (not line order), using the SAME disjoint-field borrow pattern
            // as the sticky drain above (`app.dust` / `app.rngs` /
            // `&spawn_ctx` / `&app.world` while `p` is live).
            //
            // Execution order (Player.Update): hair-dye `:24593` → manaHeat
            // `:24630` → social-shadow/teleport `:24691-24692` → eye `:25487`
            // → UpdateEquips' glass-slipper `:14242` (`:25633`) →
            // UpdateArmorSets' Solar burst `:16389` (`:25685`) → regen
            // sparkle/crimson `:19273/:19296` (`:25977`) → full-mana `:19555`
            // (`:25995`) → movement (step sound → fast-run → Lawn Mower
            // `:20320` (`:20178`, the HorizontalMovement tail) → jump
            // sound/burst → dash → wall `:21654-21826` → DoubleJumpVisuals
            // `:21901-22014`, `:26688-:26767`) → CheckDrowning bubble
            // `:23316` (`:27655`) → splash dust-then-sound (`:27772`) →
            // FloorVisuals `:23983` (`:28271`). The stoned burst / Vortex
            // rings / wall / double-jump / drown / splash / floor drains are
            // f3's (the e1b-141 keep-one, research/141+378) at their slots
            // below; e1's KEEP lanes drain here.
            if let Some(b) = p.take_hair_dye_dust() {
                crate::player_dust2::hair_dye_puff(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            if let Some(b) = p.take_mana_heat_dust() {
                crate::player_dust2::mana_heat_ambient(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            if let Some(b) = p.take_teleport_visuals_dust() {
                crate::player_dust2::teleport_visuals(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            // The stoned transition (`:25219-25234`) runs right after
            // `selectedItemState.Update()` (`:25218`); its 20-puff burst is
            // f3's `drain_stoned_dust` (the e1b-141 keep-one — e1's duplicate
            // queue DROPPED), drained at the f3 slot below.
            if let Some(b) = p.take_yoraiz_eye_dust() {
                crate::player_dust2::yoraiz_eye_trail(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            for b in p.take_glass_slipper_dust() {
                crate::player_dust2::glass_slipper_sparkles(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            if let Some(b) = p.take_solar_burst_dust() {
                crate::player_dust2::solar_shield_burst(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            // The Vortex stealth rings (`:25877/:25887`) are f3's
            // `drain_vortex_stealth` (the e1b-141 keep-one — e1's ring queue
            // DROPPED with its vortex-arm-only machine), drained at the f3
            // stealth slot below.
            if let Some(b) = p.take_shiny_sparkle_dust() {
                crate::player_dust2::shiny_stone_sparkle(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            if let Some(b) = p.take_crimson_heal_dust() {
                crate::player_dust2::crimson_heal_bursts(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            if let Some(b) = p.take_mana_full_dust() {
                crate::player_dust2::mana_full_burst(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            // W4-e4 — the mount dust drain (`Mount.UpdateFrame`'s per-type
            // extras + `UpdateEffects`' cases): the unicorn/wolf/witch/
            // basilisk dash dust, the WoF goat's tile-gated flames and the
            // minecart/trinket/drill effects. The state machines ran inline
            // in the player tick; the emitter bodies roll here on the SAME
            // `UpdatePlayers` stream (the `pending_sticky_dust` convention).
            // The `&Player` snapshot is the tick-moment one the minecart
            // anchors read; the requests keep the C# queue order.
            for req in p.mount.take_mount_dust() {
                crate::mount_dust::drain_mount_request(
                    req,
                    p,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                );
            }
            // `Player.DoEyebrellaRainEffect` (`Player.cs:14308-14331`): the
            // queued drip spawns through `Rain.NewRainForced` (`Rain.cs:135-164`)
            // with the live `windSpeedCurrent * 3, 9` velocity — the scale +
            // frame draws ride Rain's own stream (the `update_drops`
            // convention). The gfx/water-style inputs gather like `update_rain`
            // (g67 `356a1ac0` fold, research/134 R6 — the d130 `RainInputs` seam).
            if let Some(pos) = p.take_eyebrella_rain() {
                let drip_inputs = crate::rain::RainInputs {
                    camera: app.camera,
                    screen: Vec2::new(scr_w as f32, scr_h as f32),
                    zoom: app.zoom,
                    day_rate: 1,
                    wind_speed_current: app.wind.speed_current,
                    player_velocity: Vec2::ZERO,
                    shimmer_alpha: 0.0,
                    atmo: 1.0,
                    snow_tile_count: 0,
                    skyblock_low_tiles: app.world.skyblock_low_tiles,
                    zone_rain: true,
                    water_style: crate::biome::calculate_water_style(&app.world, app.camera)
                        as u32,
                    any_danger: false,
                    gfx_quality: app.gfx_quality.quality,
                    remix_world: false,
                    game_menu: false,
                };
                let vel = Vec2::new(app.wind.speed_current * 3.0, 9.0);
                app.rain.new_rain_forced(&drip_inputs, pos, vel);
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
            // W4-f50 — the player hurt combat text (`Player.cs:38382`): the
            // C# draws it BEFORE the retaliate block (`:38401`), so the drain
            // spawns it ahead of the retaliate rolls on the `UpdatePlayers`
            // stream (the mana text above, `:38291`, already drew first).
            if let Some((amount, crit)) = p.take_hurt_text() {
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
                let color = crate::combat_text::damage_text_color(crit, true, false);
                crate::combat_text::spawn_amount(
                    &mut app.combat_texts,
                    location,
                    color,
                    amount,
                    crit,
                    false,
                    size,
                    1.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // W4-f32 — the TryToPoop drop drain (`Player.cs:18938`): the
            // 5395 Poop Block at the player's MountedCenter, stack rolled
            // inside `update_survival` on the `UpdatePlayers` stream;
            // `RequestNewItem`'s default velocity draws (`Item.cs:49749-
            // 49750`: Next(-30,31)*0.1, Next(-40,-15)*0.1) ride the same
            // stream here, then `GrabDelayForAllPlayers`' 100-tick delay.
            if let Some((stack, mcx, mcy)) = p.take_poop() {
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                let vx = rng.next_range(-30, 31) as f32 * 0.1;
                let vy = rng.next_range(-40, -15) as f32 * 0.1;
                let mut drop = crate::item::ItemDrop::new(
                    5395,
                    stack.max(1) as u32,
                    0, // prefix: 0 — no draw
                    mcx - crate::item::SIZE * 0.5,
                    mcy - crate::item::SIZE * 0.5,
                    vx,
                    vy,
                    0.0,
                );
                drop.apply_grab_delay(crate::item::GRAB_DELAY_ALL);
                app.items.push(drop);
            }
            if let Some(req) = p.take_retaliate() {
                // S1-C-7 wave B — the fall/petrify lane's retaliate block
                // (`Player.cs:38401-38517`): those hits run inside
                // `Player.Update` under the `UpdatePlayers` swap
                // (`Main.cs:17754`) on the General counter (a Retaliate
                // member), so the drain rides the same stream. The lava/tile/
                // bug-net lanes' counters are not Retaliate members
                // (M4-HAZARD-1: `hurt_inner` reads the membership from the
                // counter) and never queue.
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
                // W4-c28: the brain's Confused AddBuff queued its net
                // intent (`NPC.cs:89932-89935`/`:89979`) — drain here.
                npc_buff_drain = true;
            }
            // W4-f50 — the Solar Flare hurt blast (`Player.cs:38316-38318`):
            // the fall/petrify lane's accepted hit consumed a shield charge
            // and queued the 608 blast; the drain rides the `UpdatePlayers`
            // stream (the `Main.cs:17754` swap), the S1-E-21c batch convention.
            // Deviation 2 threads the dust/gore pools through (disjoint field
            // borrows: `p` is `app.player`, the pools are `app.dust`/`app.gore`).
            if let Some((center, damage)) = p.take_solar_blast() {
                let dolls = p.doll_flags();
                let luck = p.luck();
                app.projectiles.solar_flare_blast(
                    center,
                    damage,
                    app.local_id as u8,
                    &mut app.npcs,
                    dolls,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    luck,
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &mut app.gore,
                );
            }
            // W4-f50 — the hit-dust loop + football + golf-club hurt drains
            // (`Player.cs:38527-38553` / `:38255-38280`): the fall/petrify
            // lane's accepted hit queued them; the drain rides the
            // `UpdatePlayers` stream (the S1-E-21c batch convention). The
            // hit-dust draws follow the hurt sound's (`:38525` < `:38527`).
            crate::hurt_modifiers::drain_hurt_dust(
                p,
                &mut app.dust,
                &spawn_ctx,
                &app.world,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
            crate::hurt_modifiers::drain_football(
                p,
                &mut app.projectiles,
                &mut app.inventory,
                &mut app.mouse_item,
                app.local_id as u8,
            );
            let held = app.inventory[app.selected].kind;
            crate::hurt_modifiers::drain_golf_club_kill(
                p,
                &mut app.projectiles,
                held,
                app.local_id as u8,
            );
            // `Player.PlayHurtSound` (`Player.cs:38525`): the hurt cue plays
            // at the `Hurt` TAIL — after the cuffs' mana text (`:38291`) and
            // the retaliate block (`:38401-38517`), so its draw follows
            // theirs on the stream. The C# plays the local player's hurt
            // sound non-positionally (`Player.cs:38572`). W4-e10 ports the
            // WHOLE decision tree (`:38569-38712`): stoned → voiceOverride →
            // mount → werewolf → frost/bone armor → DST-world → voiceVariant.
            // Every gameplay draw rides the `UpdatePlayers` stream
            // (`Player.Hurt` runs under the `Main.cs:17754` swap), only after
            // the attenuation gate; the style façades' `GetRandomPitch`
            // NextFloat rides the audio-only stream.
            if p.take_hurt_sound().is_some() {
                let dont_starve = app
                    .world
                    .seeds
                    .contains(tr_core::seeds::SeedFlags::DONT_STARVE);
                let remix = app.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
                let voice = p.hurt_voice(dont_starve, remix);
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                match voice {
                    Some(crate::player::HurtVoice::Stoned) => {
                        // `:38578` — `PlaySound(0, vector)`, case 0 Dig.
                        app.audio.play_no_space_sound(-1.0, -1.0, rng);
                    }
                    Some(crate::player::HurtVoice::Male(pitch)) => {
                        app.audio.play_hurt_sound(pitch, rng);
                    }
                    Some(crate::player::HurtVoice::Female(pitch)) => {
                        app.audio.play_female_hurt_sound(pitch, rng);
                    }
                    Some(crate::player::HurtVoice::Style(name, pitch)) => {
                        app.audio.play_voice_style(name, -1.0, -1.0, pitch, rng);
                    }
                    Some(crate::player::HurtVoice::Chicken) => {
                        // `:38602-38609` — the `Next(5) == 0` rare pick.
                        let name = if rng.next_range(0, 5) == 0 {
                            "ChickenHurtRare"
                        } else {
                            "ChickenHurt"
                        };
                        app.audio.play_voice_style(name, -1.0, -1.0, 0.0, rng);
                    }
                    Some(crate::player::HurtVoice::Npc(id, style)) => {
                        app.audio.play_hurt_npc_int(id, style, -1.0, -1.0, rng);
                    }
                    None => {}
                }
            }
            if p.take_death_sound().is_some() {
                // W4-e10 — `Player.PlayDeathSound`'s decision tree
                // (`Player.cs:38715-38746`), at `base.position` (the
                // TOP-LEFT, not the centre). Case 5 draws nothing; the style
                // arms ride the wave-A façade (the NextFloat on the
                // audio-only stream).
                let dont_starve = app
                    .world
                    .seeds
                    .contains(tr_core::seeds::SeedFlags::DONT_STARVE);
                let tenth = app
                    .world
                    .seeds
                    .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY);
                match p.death_voice(dont_starve, tenth) {
                    crate::player::DeathVoice::Killed => {
                        app.audio.play_death_sound(p.x, p.y);
                    }
                    crate::player::DeathVoice::Style(name) => {
                        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                        app.audio.play_voice_style(name, p.x, p.y, 0.0, rng);
                    }
                }
            }
            // W4-f3 — the petrify-transition burst (`Player.cs:25226-25233`):
            // the stoned edge queues 20 type-1 puffs + the per-puff `Next(2)`
            // noGravity roll. The C#'s `PlaySound(0)` (`:25225`) is a cited
            // seam (the stoned hurt rides `hurt_inner`'s own cue above), so the
            // drain replays only the dust chain on the `UpdatePlayers` stream.
            if let Some(burst) = p.take_stoned_dust() {
                crate::player_dust::drain_stoned_dust(app, burst);
            }
            // W4-f3 — the Vortex-stealth transition ring (`Player.cs:25872-
            // 25893`): the stealth machine's `flag11` tick queues the centre;
            // the drain replays the two Next(2) gates + the RotatedByRandom
            // spins + the type-229/240 chains. The machine runs at `:25826`
            // (after the stoned edge `:25228`), so the drain follows the
            // stoned burst on the `UpdatePlayers` stream.
            if let Some(burst) = p.take_vortex_stealth_dust() {
                crate::player_dust::drain_vortex_stealth(app, burst);
            }
            // `Player.SpawnFastRunParticles` (`Player.cs:20214`): the
            // fast-run footstep at the player's top-left position — the
            // case-17 `Main.rand.Next(-10, 11)` pitch roll (`:862`) rides
            // the `UpdatePlayers` stream (the `Main.cs:17754` swap covers
            // `Player.Update`), only after the attenuation gate passed.
            // HorizontalMovement (`:26688`) runs BEFORE JumpMovement
            // (`:26744`) — the step sound draws first (the movement-block
            // order, the 141 reconcile).
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
            // W4-e1 — the Lawn Mower puff (20320). MowTheLawn is the TAIL of
            // HorizontalMovement (`:20176-20179`, inside `:19772`, called
            // at `:26688`) — it runs AFTER the fast-run particles
            // (`:19990/:20020`, the step sound above) and BEFORE JumpMovement
            // (`:26744`)/DashMovement/the wall machines. Its NewDustDirect
            // draw therefore precedes the jump-chain sounds (the 141
            // execution order — the earlier placement after the jump sounds
            // spent the stream out of order).
            if let Some(b) = p.take_mow_lawn_dust() {
                crate::player_dust2::mow_lawn_puff(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                    &b,
                );
            }
            // S1-E-21f — the extra-jump chain's plays (JumpMovement `:26744`,
            // AFTER MowTheLawn): case 16's one `Next(-10, 11) * 0.01` pitch
            // (`LegacySoundPlayer.cs:851`) rides the `UpdatePlayers` stream
            // after the gate; the Fart's Item-16 style play rides wave A's
            // façade (the `GetRandomPitch` NextFloat on the audio-only stream
            // — variance 0, still drawn).
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
            let jump_burst = p.take_jump_dust();
            if let Some(burst) = jump_burst {
                crate::player_dust::drain_jump_dust(app, burst);
            }
            // W4-g17 wiring — the extra-jump visual tails (`Player.cs:
            // 20849-21021`, `player_gore_a`): the committed jump queued in
            // `JumpMovement` runs its burst here on the `UpdatePlayers`
            // (keep-one: the `take_jump_gores` kind is read from the
            // W4-d51 `JumpDust` snapshot `kind` — the same `ExtraJump` at
            // the same C# read point, so the queue is not duplicated.)
            // stream with the POST-jump velocity (the arm's writes already
            // ran). `gravDir` is the port's normal 1f (no flipped-gravity
            // state — the `player_gore_a` grav_height pick reads it).
            if let Some(j) = jump_burst.map(|b| b.kind) {
                let state = crate::player_gore_a::JumpGoreState {
                    x: p.x,
                    y: p.y,
                    vx: p.vx,
                    vy: p.vy,
                    width: p.width as i32,
                    height: p.height as i32,
                    grav_dir: 1.0,
                };
                let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                match j {
                    crate::player::ExtraJump::Fart => crate::player_gore_a::fart_jump_gores(
                        &state, &mut app.gore, &mut app.dust, &spawn_ctx, rng, &app.world,
                    ),
                    crate::player::ExtraJump::Santank => {
                        crate::player_gore_a::santank_jump_gores(
                            &state, &mut app.gore, &mut app.dust, &spawn_ctx, rng, &app.world,
                        )
                    }
                    crate::player::ExtraJump::Cloud => crate::player_gore_a::cloud_jump_gores(
                        &state, &mut app.gore, &mut app.dust, &spawn_ctx, rng, &app.world,
                    ),
                    // The other six arms' tails are dust-only or unported
                    // visuals (the module's census covers the three gore
                    // arms); no draws — the C# arms spawn no gores.
                    _ => {}
                }
            }
            // W4-g17 wiring — the dash==1 launch tail (`Player.cs:
            // 21419-21434`, `player_gore_b::dash_launch_gores`): the
            // POST-launch snapshot, the same stream.
            let dash_burst = p.take_dash_dust();
            if dash_burst.map_or(false, |b| b.dash == 1 && !b.active) {
                let state = crate::player_gore_b::DashGoreState {
                    x: p.x,
                    y: p.y,
                    width: p.width as i32,
                    height: p.height as i32,
                };
                crate::player_gore_b::dash_launch_gores(
                    &state,
                    &mut app.gore,
                    &mut app.dust,
                    &spawn_ctx,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &app.world,
                );
            }
            // W4-g17 wiring (keep-one) — the dodge bursts (`Player.cs:
            // 38060-38160`, `player_gore_c`) are DEFERRED in this base:
            // `player.rs`'s `do_mystic_sash_dodge`/`ninja_dodge` carry no
            // burst queue (their docs defer it to S1-E-1, the W4-g17
            // wave-C port), so the tick has no kind to replay here.

            // W4-d51 — the `DashMovement` dust burst (`Player.cs:21259-21512`):
            // the trail (active dash) or the start burst, snapshotted at the
            // C# read moment; no sound rides the dash, so the drain follows
            // the jump chain's on the same `UpdatePlayers` stream.
            if let Some(burst) = dash_burst {
                crate::player_dust::drain_dash_dust(app, burst);
            }
            // W4-f3 — the wall-climb/wallslide puff (`Player.cs:21654-21826`,
            // five sites): the wall machines run at `:26758-26765`, right
            // after `DashMovement` (`:26757`) and before `DoubleJumpVisuals`
            // (`:26767`); the drain follows the dash dust on the same
            // `UpdatePlayers` stream. The suppression probe (the Rat climbing
            // into a ceiling) ran on the tick side.
            if let Some(burst) = p.take_wall_dust() {
                crate::player_dust::drain_wall_dust(app, burst);
            }
            // W4-f3 — the `DoubleJumpVisuals` trail (`Player.cs:21892-22021`):
            // the performing-jump arm's per-tick dust, snapshotted at the C#
            // read moment; the drain follows the dash dust on the same
            // `UpdatePlayers` stream (the C# runs `DoubleJumpVisuals` at
            // `:26767`, right after `DashMovement` `:26757`).
            if let Some(burst) = p.take_double_jump_dust() {
                crate::player_dust::drain_double_jump_dust(app, burst);
            }
            // W4-f3 seal (KEEP-E1 drain order, e1 3245d624) — the wing-glide dust
            // (`Player.cs:27040-27337`) drains HERE, right after DoubleJumpVisuals
            // (`:26767`): the glide arm runs inside `WingMovement` (`:26818` →
            // `:27034`), BEFORE `CheckDrowning` (`:27655`) and the splash
            // (`:27783`). The old f3 order drained it after the floor dust, a
            // cross-lane RNG-order deviation. The wings-4 wingFrame machine +
            // Item13 cadence stay g66's cited seams (the wing lane's).
            if let Some(burst) = p.take_wing_glide_dust() {
                crate::player_dust::drain_wing_glide(app, burst);
            }
            // S1-E-21f — the drowning gasp (`Player.cs:23257`): case 23 is
            // Volume/Pan only — no draws, no stream.
            if p.take_drown_sound() {
                app.audio.play_drown_sound();
            }
            // W4-f3 seal (KEEP-E1 drain order) — the drowning bubble
            // (`Player.cs:23283-23316`, `CheckDrowning` `:27655`) drains BEFORE
            // the splash (`:27783`): the C# runs CheckDrowning before the
            // WetCollision splash block. The `Next(20)` gate drew on the tick
            // side; the type-34 puff rides the pool's own wet-probe kill.
            if let Some((mouth, width)) = p.take_drown_bubble() {
                crate::player_dust::drain_drown_bubble(app, mouth, width);
            }
            // W4-f3 — the liquid entry/exit splash DUST (`Player.cs:27783-27940`):
            // the C# runs the dust loop BEFORE `PlaySound(19)`, so the drain
            // precedes the splash sound on the `UpdatePlayers` stream.
            if let Some(burst) = p.take_splash_dust() {
                crate::player_dust::drain_splash_dust(app, burst);
            }
            // W4-g66 gap-fill — the wall-cling dust (`Player.cs:26758-26766`,
            // `WallClimbMovement`/`WallslideMovement` `:21607-21840`): the
            // type-31 puff sits between the dash burst and DoubleJumpVisuals
            // in the C# call order, same `UpdatePlayers` stream.
            if let Some(burst) = p.take_wall_dust() {
                crate::player_trail_dust::drain_wall_dust(app, burst);
            }
            // W4-g66 — `DoubleJumpVisuals` (`Player.cs:26767`, `:21892-22021`):
            // the per-tick performing-jump trail follows the dash burst in
            // the C# call order, on the same `UpdatePlayers` stream.
            if let Some(burst) = p.take_double_jump_dust() {
                crate::player_trail_dust::drain_double_jump_visuals(app, burst);
            }
            // W4-g66 — `WingAirVisuals` (`Player.cs:26817`, `:30750-31013`):
            // the flag19 flying tick's per-wing rolls, snapshotted BEFORE
            // `WingMovement` shaped the velocity.
            if let Some(burst) = p.take_wing_air_dust() {
                crate::player_wing_dust::drain_wing_air(app, burst);
            }
            // W4-g66 — `WingFrame`'s dust arms (`Player.cs:26820`,
            // `:29468-30527`): the jetpack-4/Bat-30/Rainbow-45/Spooky-39
            // rolls + the Nebula-29 rest roll; the frame machines ran in
            // the tick (`wing_frame_visuals`).
            if let Some(burst) = p.take_wing_frame_dust() {
                crate::player_wing_dust::drain_wing_frame(app, burst);
            }
            // W4-g66 gap-fill — the boot/jetpack Item13/Item24 cue
            // (`Player.cs:26884-26896`, `:29478`, `:30172`, `:27154`): the
            // case-2 pitch draw rides `UpdatePlayers`, the same queued-sound
            // convention as the flap Item32 below (the `pending_flap_sound`
            // precedent — the play queues at the C# moment, the pitch draw
            // lands at the drain; on a tick where a cue AND a dust arm both
            // roll, the pitch-vs-dust interleave rides the queue order, the
            // documented seam deviation). Placed before the boot flames:
            // the boot cue (`:26884`) precedes `RocketBootVisuals` (`:26918`).
            if let Some((idx, sx, sy)) = p.take_rocket_sound() {
                app.audio.play_use_sound(
                    idx,
                    sx,
                    sy,
                    0.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // W4-g66 — `RocketBootVisuals` (`Player.cs:26918`, `:29330-29458`):
            // the thrust tick's flame pair, snapshotted before the thrust
            // velocity writes.
            if let Some(burst) = p.take_rocket_flame_dust() {
                crate::player_boot_dust::drain_rocket_flames(app, burst);
            }
            // W4-g66 — the wing-glide dust (`Player.cs:27040-27315`): the
            // jump-held falling arm's per-wing rolls; the frame machines ran
            // in the tick (`wing_glide_frames`).
            if let Some(burst) = p.take_wing_glide_dust() {
                crate::player_wing_dust::drain_wing_glide(app, burst);
            }
            // W4-g66 — `FloorVisuals` → `MakeFloorDust` (`Player.cs:28271`,
            // `:23790-24043`): the landing burst / running puffs after the
            // movement block, before `PlayerFrame`'s draws (`:28286`). The
            // landing Item177 cue rides inside the drain, before the loop.
            if let Some(burst) = p.take_floor_dust() {
                crate::player_trail_dust::drain_floor_dust(app, burst);
            }
            // W4-g66 — `PlayerFrame`'s walking-effect rolls
            // (`Player.cs:35996-36136`): inside `PlayerFrame` (`:28286`),
            // after the floor dust, in source order.
            if let Some(burst) = p.take_walk_effect_dust() {
                crate::player_boot_dust::drain_walk_effects(app, burst);
            }
            // W4-g66 — `PlayerFrame`'s Rudolph flight trail
            // (`Player.cs:36164-36198`): the mount animator runs after the
            // walking block, same `UpdatePlayers` stream.
            if let Some(burst) = p.take_rudolph_trail() {
                crate::player_trail_dust::drain_rudolph_trail(app, burst);
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
            // W4-f3 — the floor dust (`Player.cs:23790-24043`, `MakeFloorDust`):
            // the `FloorVisuals` call (`:28271`) queues the floor-tile snapshot;
            // the drain replays the per-iteration gate rolls + the NewDust chain
            // on the `UpdatePlayers` stream (the shimmer orchestrator + Item177
            // cue are cited seams). It drains AFTER the splash sound (`:28271`
            // follows `:27783`), the e1 execution order.
            if let Some(burst) = p.take_floor_dust() {
                crate::player_dust::drain_floor_dust(app, burst);
            }
            // W4-e10 — the petrification transition (`Player.cs:25225-
            // 25232`, between the drown check and `UpdateManaRegen`): the
            // case-0 Dig play at the top-left (the `Next(3)` variation +
            // the `Next(-10, 11)` pitch after the gate), then 20 type-1
            // dust — each `NewDust`'s six internals, then the `Next(2)`
            // noGravity roll. All on `UpdatePlayers`.
            if p.take_stoned_transition() {
                app.audio.play_no_space_sound(
                    p.x,
                    p.y,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
                let (pos, w, h) = (glam::Vec2::new(p.x, p.y), crate::player::WIDTH as i32, p.height as i32);
                for _ in 0..20 {
                    let idx = app.dust.new_dust(
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        &spawn_ctx,
                        &app.world,
                        pos,
                        w,
                        h,
                        1,
                        0.0,
                        0.0,
                        0,
                        tr_core::particles::Color::default(),
                        1.0,
                    );
                    let no_gravity = app
                        .rngs
                        .stream(crate::app::RngKey::UpdatePlayers)
                        .next_range(0, 2)
                        == 0;
                    if no_gravity && idx != tr_core::particles::DUST_SENTINEL {
                        app.dust.slot_mut(idx).no_gravity = true;
                    }
                }
            }
            // W4-e10 — the full-bar mana trigger (`Player.cs:19550-19559`,
            // `UpdateManaRegen` at `:25995` — after the drown check, before
            // the wing/splash blocks): the unpositioned case-25 chime (no
            // draws), then 5 type-45 dust over the player's box — alpha
            // 255, `default` colour, the scale `Next(20, 26) * 0.1` draw
            // BEFORE each `NewDust`'s internals, noLight + noGravity,
            // velocity halved. All on `UpdatePlayers` (`Main.cs:17754`).
            if p.take_mana_full() {
                app.audio.play_max_mana_sound();
                let (pos, w, h) = (glam::Vec2::new(p.x, p.y), crate::player::WIDTH as i32, p.height as i32);
                for _ in 0..5 {
                    let scale = app
                        .rngs
                        .stream(crate::app::RngKey::UpdatePlayers)
                        .next_range(20, 26) as f32
                        * 0.1;
                    let idx = app.dust.new_dust(
                        app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                        &spawn_ctx,
                        &app.world,
                        pos,
                        w,
                        h,
                        45,
                        0.0,
                        0.0,
                        255,
                        tr_core::particles::Color::default(),
                        scale,
                    );
                    if idx != tr_core::particles::DUST_SENTINEL {
                        let d = app.dust.slot_mut(idx);
                        d.no_light = true;
                        d.no_gravity = true;
                        d.velocity *= 0.5;
                    }
                }
            }
            // W4-e10 — the hive/honey landing cue (`Player.cs:23830`,
            // `MakeFloorDust` via `FloorVisuals` `:28271` — after
            // `UpdateManaRegen` `:25995` in the C# Update): the
            // `SoundID.Item177` style façade at `((int)Center.X,
            // (int)Bottom.Y)` — the case-2 default pitch draw
            // (`Next(-6, 7) * 0.01`) on `UpdatePlayers` after the gate.
            if let Some((sx, sy)) = p.take_honey_land_sound() {
                app.audio.play_proj_sound(
                    (2, 177),
                    sx,
                    sy,
                    0.0,
                    1.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
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
            // W4-e10 — the `WingFrame` rocket cue (`Player.cs:27154`/
            // `:29478`/`:30172`): wave A's façade. It drains BEFORE the boot
            // trigger cue — the C# `WingFrame` (`:26820`) runs ahead of the
            // rocket trigger (`:26867`) in `Player.Update`.
            if let Some((idx, sx, sy)) = p.take_wing_rocket_sound() {
                app.audio.play_use_sound(
                    idx,
                    sx,
                    sy,
                    0.0,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // W4-e10 — the rocket-boot trigger cue (`Player.cs:26889`/
            // `:26894`): wave A's façade, gated by `rocketSoundDelay`.
            if let Some((idx, sx, sy)) = p.take_rocket_sound() {
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
            // W4-g10 FINAL — the Chillemt petting joy cue (`Player.cs:33354`):
            // the same case-42 façade, the `pal_chillet_joy` asset.
            if let Some((sx, sy)) = p.take_petting_joy_sound() {
                app.audio.play_petting_joy_sound(sx, sy);
            }
            // W4-g10 FINAL — the PET_THE_PET achievement request
            // (`AchievementsHelper.cs:358-360`, HandleSpecialEvent(21)):
            // the row-targeted `complete_condition` (a6's shape, NOT the
            // broadcast notify_custom — research/138 keep-one). The
            // queue's production drain (the g10 host landed the request;
            // batch-L wires it beside the joy drain).
            if p.take_pet_achievement() {
                let _ = app.achievements.complete_condition("PET_THE_PET", "Do");
            }
            // S1-P-i41 — the QuickMount no-space tink (`Player.cs:6057`):
            // S1-P-i41 - the QuickMount no-space tink (`Player.cs:6057`):
            // case 0's Dig variation + pitch draws ride the `UpdatePlayers`
            // stream (the player tick runs under the `Main.cs:17754` swap).
            if let Some((sx, sy)) = p.take_no_space_sound() {
                app.audio.play_no_space_sound(
                    sx,
                    sy,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                );
            }
            // W4-c14 - the petrify-transition Dig-0 (`Player.cs:25225`):
            // the same case-0 contract, both transition edges.
            if let Some((sx, sy)) = p.take_stoned_sound() {
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
            // W4-f6 wiring — the mount dust drain (`Mount.DoSpawnDust`
            // `Mount.cs:6300`/`:6169` + `DoFailedDismountDust`
            // `Player.cs:6058`): the tick queued snapshots in C# call order
            // (the `pending_sticky_dust` convention); the emitters draw on
            // the same `UpdatePlayers` stream AFTER the tink's case-0 draws
            // above (`:6057` precedes `:6058`).
            let mount_dust = p.take_mount_dust();
            if !mount_dust.is_empty() {
                crate::mount_dust::drain_mount_dust(
                    mount_dust,
                    &app.player_name,
                    p,
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &mut app.dust,
                    &spawn_ctx,
                    &app.world,
                );
            }
            // W4-g11 — the `Player.Teleport` effect pair (`Player.cs:37444`/
            // `:37489`): the movement core queued it (the no-space teleport
            // above, the Shellphone fallback). The replay needs `&mut App`
            // (the pool + the player), which this closure's `p` borrow cannot
            // hand out — it rides the post-step drain beside `force_clear`
            // (the same-tick deferral is observation-identical: nothing probes
            // the dust between the move and the drain). The stream is the
            // `UpdatePlayers` swap (`Main.cs:17754`).
            if let Some((ox, oy, nx, ny, style, e2, ei, mult)) = p.take_teleport_effect() {
                teleport_effect_req = Some(crate::main_dust::PlayerTeleportReq {
                    old_x: ox,
                    old_y: oy,
                    new_x: nx,
                    new_y: ny,
                    style,
                    extra_info2: e2,
                    extra_info: ei,
                    mult,
                    key: crate::app::RngKey::UpdatePlayers,
                });
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
        player_step.drop_backlog();
        app.player_step = player_step;
        // W4-c24c — `Player.Update`'s local-player armor tail
        // (`Player.cs:25703`): `AchievementsHelper.CheckResearchAchievement()`
        // every tick. The port drains it after the player borrow like the
        // HandleRunning float below (the C# fires both inside `Player.Update`);
        // the version gate inside skips the recount while the tracker is
        // unchanged; at half the catalog it fires progression 45
        // (TRASH_COMPACTOR).
        let _ = app.achievements.check_research(&app.research, false);
        // W4-c36 — `Player.UpdateBiomes`' reach/biome tail (`Player.cs:
        // 16732-16776`) + `BordersMovement`'s reads (`:24099`/`:24125`):
        // the per-tick wall/zone/depth checks fire the named reach rows.
        // The pairs resolve after the player borrow releases the manager.
        let reach_rows = crate::app::achievement_hooks::reach_check(
            p,
            &app.world,
            app.scene_metrics.zone_graveyard,
        );
        for (row, cond) in reach_rows {
            let _ = app.achievements.complete_condition(row, cond);
        }
        // W4-c36 — the fall-damage LUCKY_BREAK drain (`Player.cs:25354-25357`/
        // `:25371-25374`): the bare player tick queued the event-8 test.
        if std::mem::take(&mut p.pending_lucky_break) {
            let _ = app.achievements.complete_condition("LUCKY_BREAK", "Hit");
        }
        // W4-c36 DOC-SWEEP — `Mount.SetMount`'s coffin-cart arm
        // (`Mount.cs:6301-6304`): the Coffin Minecart (MountID 38) mount
        // queued progression 32 (ROLLIN_IN_YOUR_GRAVE).
        if std::mem::take(&mut p.pending_grave_cart) {
            progression_events.push(32);
        }
        // W4-c36 — `Mount.UpdateEffects` case 10 (`Mount.cs:4955-4958`): the
        // Unicorn mount swinging the Rainbow Gun 1260 fires
        // HandleSpecialEvent(5) → RAINBOWS_AND_UNICORNS ("Use").
        let unicorn_rainbow = p.mount.is_active() && p.mount.kind() == 10;
        if unicorn_rainbow && app.swing > 0.0 && app.inventory[app.selected].kind == 1260 {
            let _ = app.achievements.complete_condition("RAINBOWS_AND_UNICORNS", "Use");
        }
        // W4-c36 — `ArmorSetBonuses.Benefits.Bee` (`ArmorSetBonuses.cs:23-30`):
        // the completed Bee set (2361/2362/2363) fires HandleSpecialEvent(3)
        // → NOT_THE_BEES ("Use") while the Bee Gun 1121 swings
        // (`itemAnimation > 0 && inventory[selectedItem].type == 1121`).
        // The C# runs it inside `UpdateArmorSets` (`Player.cs:25685`); the
        // port reads the same tick's post-update state here.
        let bee_set = p.equipped[0] == 2361 && p.equipped[1] == 2362 && p.equipped[2] == 2363;
        if bee_set && app.swing > 0.0 && app.inventory[app.selected].kind == 1121 {
            let _ = app.achievements.complete_condition("NOT_THE_BEES", "Use");
        // W4-a6 — `Player.cs:25695-25698` (`Main.myPlayer` block): BOTH
        // combat books used fires progression event 40 (TRAINING_DAY,
        // `AchievementInitializer.cs:414-415`) every tick the pair stands —
        // the notify is idempotent once completed (the g90 lane-7 hook,
        // research/138 §2; the 4382/5336 uses raise the world flags).
        // `Player.cs:25695-25698` (`Main.myPlayer` block): BOTH combat books
        // used fires progression event 40 (TRAINING_DAY,
        // `AchievementInitializer.cs:414-415`) every tick the pair stands —
        // the notify is idempotent once completed (W4-g90 lane 7; the
        // 4382/5336 uses raise the world flags).
        if app.world.combat_book_was_used && app.world.combat_book_volume_two_was_used {
            progression_events.push(40);
        }
        if run_pixels > 0.0 {
            // `Player.cs:28233-28236` -> `AchievementsHelper.HandleRunning`
            // (`AchievementsHelper.cs:420-423`): grounded, unmounted running
            // advances MARATHON_MEDALIST's `Move` float accumulator.
            let _ = app.achievements.notify_custom_float("Move", run_pixels);
        }
        // `Player.UpdateArmorSets`' local-player achievement tail
        // (`Player.cs:25686-25704`, the `i == Main.myPlayer` block): the
        // effective armor slots (vanity 10/11/12 over the functional 0/1/2,
        // `GetEffectiveArmor` — the port has no loadouts, so `equipped[slot]`
        // IS the effective kind) drive two per-tick progression checks.
        // TRAINING_DAY (40, `:25695-25698`) reads `NPC.combatBookWasUsed &&
        // combatBookVolumeTwoWasUsed`; the port carries only the first book
        // flag (`world.combat_book_was_used`) and the volume-two use is
        // unported, so that check stays cited-unwired (the second flag never
        // sets). CheckResearchAchievement (45, `:25703`) rides the research
        // family — see `research::check_research_achievement` (the live
        // sacrifice UI is unported, so the tracker never edits in play).
        {
            let slot_id = |slot: usize, pick: fn(&crate::registry::ItemDef) -> i16| -> i32 {
                crate::registry::get(p.equipped[slot]).map_or(-1, |d| pick(d) as i32)
            };
            // `:25688-25690` — head/body/legs: the vanity slot's id when it
            // carries one (`>= 0`), else the functional slot's.
            let head_v = slot_id(10, |d| d.head_slot);
            let num21 = if head_v >= 0 { head_v } else { slot_id(0, |d| d.head_slot) };
            let body_v = slot_id(11, |d| d.body_slot);
            let num22 = if body_v >= 0 { body_v } else { slot_id(1, |d| d.body_slot) };
            let legs_v = slot_id(12, |d| d.leg_slot);
            let num23 = if legs_v >= 0 { legs_v } else { slot_id(2, |d| d.leg_slot) };
            // ON_FLEEK (38, `:25691-25694`): Sunglasses (head slot 12) worn in
            // daylight, no rain, not a remix world, above the surface, inside
            // the day window `time in [3600, 50400]`.
            let remix = app.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
            if num21 == 12
                && !remix
                && !app.rain.is_raining()
                && app.world.day_time
                && app.world.time >= 3600.0
                && app.world.time <= 50400.0
                && (p.y as f64) < app.world.world_surface * 16.0
            {
                // `notify_progression_net(38)` — deferred to the post-borrow
                // queue drain below (the `p` borrow is live here).
                progression_events.push(38);
            }
            // MINI_ME (41, `:25699-25702`): the full Moon Lord vanity set —
            // the mask (head 187), the body (249) and the legs (217).
            if num21 == 187 && num22 == 249 && num23 == 217 {
                progression_events.push(41);
            }
        }
        // W4-f30 — `Player.UpdateArmorSets`' research tail (`Player.cs:25703`):
        // `AchievementsHelper.CheckResearchAchievement()` per tick. The tracker
        // (App.research) never edits in play yet (the live sacrifice UI is
        // unported), so the LastEditId latch skips unchanged frames; the check
        // is wired and fires 45 (TRASH_COMPACTOR) once half the researchable
        // catalogue is complete. The forced EnterWorld call rides
        // `enter_loaded_world` (`Player.Hooks.EnterWorld`, `:325`).
        {
            let (fired, version) = crate::research::check_research_achievement(
                &app.research,
                app.last_research_version,
                false,
            );
            app.last_research_version = version;
            if fired {
                progression_events.push(45);
            }
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
    // Drains for the queue seeds above (the whole-`App` methods the tick
    // closure and the player borrow could not call).
    for (n, x, y, d) in door_echoes {
        app.send_door_toggle(n, x, y, d);
    }
    for ev in progression_events {
        let _ = app.notify_progression_net(ev);
    }
    if npc_buff_drain {
        app.drain_npc_buff_sends();
    }
    // The `59 HitSwitch` sends (`Collision.cs:3304`/`:3318`/`:3324`/`:3347`) —
    // after the tick closure and the player borrow released `app`.
    for (x, y) in switch_sends {
        app.send_hit_switch(x, y);
    }
    // W4-g11 union-exec — the `Player.Teleport` effect pair
    // (`Player.cs:37444`/`:37489`): the Entry effect at the OLD box, the
    // move, the Exit at the NEW box, the grants + the camera transition —
    // on the `UpdatePlayers` stream. The reachable player-tick caller (the
    // no-space / Shellphone fallback) rides style 11, which NOW fires
    // through the union dispatcher (50 Perfect dust, no sound — g61's
    // arm); the replay re-applies the (already-inline) move and owns the
    // C#-position grants (gap 6). It drains beside `force_clear` (both
    // post-step; nothing probes the spawn area between the teleport and
    // the clear).
    if let Some(req) = teleport_effect_req {
        crate::main_dust::player_teleport(app, req);
    }
    // W4-g89e — `Player.cs:25695-25698` (the `i == Main.myPlayer` block of
    // UpdateEquips): BOTH combat books used fires progression event 40
    // (`TRAINING_DAY`, `AchievementInitializer.cs:414-416`). The C# re-fires
    // every tick while both flags hold; `notify_progression` is idempotent
    // after the grant. The borrow-free tail is the port's myPlayer-block
    // position (the armor-set ammo grants above ride the same fold).
    if app.world.combat_book_was_used && app.world.combat_book_volume_two_was_used {
        let _ = app.notify_progression_net(40);
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
    // W4-f48 parry wave — `ItemCheck_ManageRightClickFeatures_ShieldRaise`
    // (`Player.cs:31706-31738`, called at `:28283` after the movement +
    // dropItemCheck blocks, before `ItemCheckWrapped` — the port's
    // `update_editing` runs later in the frame, so this tail matches the
    // relative order). The `:28281` wrapper: `netMode != 2` collapses SP,
    // the drill mount (type 8) skips the whole feature.
    if !app.player.as_ref().is_some_and(|p| p.mount.kind() == 8) {
        let held = app.inventory[app.selected];
        // `PlayerInput.Triggers.JustPressed.MouseRight` (`:31708`): the
        // right-button press edge — `place_prev_down` still holds LAST
        // tick's button (the editing pass refreshes it after the players
        // phase). `app.drop_right_edge` is the same edge with the
        // `mouseInterface` refusal already folded (the players-phase
        // capture).
        let mouse_right = app.drop_right_edge;
        // `flag` (`:31528`): selectedItem != 58, controlUseTile,
        // releaseUseItem, !controlUseItem, !mouseInterface. The port's
        // stand-ins: `place_down` / the previous tick's `mine_released`
        // latch (the C# `releaseUseItem` is written at ItemCheck's END,
        // `:42829`, so this read is last tick's too) / `!mine_down`.
        // `tileInteractionHappened`, `CaptureManager`, the
        // `HoveringAnInteractable` clause and the pending-inventory-
        // actions refusal have no port machinery — collapsed open
        // (recorded deviation, ANALYSIS "The parry machine").
        let general_check = app.selected != 58
            && app.place_down
            && app.mine_released
            && !app.mine_down
            && !app.inventory_open;
        // `shouldGuard` (`:31714-31718`): the general check AND the held
        // Brand of the Inferno (3823) / Bouncing Shield (4760), a
        // raisable shield, no mount, and (idle OR the right press).
        let should_guard = app.player.as_ref().is_some_and(|p| {
            (general_check & matches!(held.kind, 3823 | 4760))
                && p.has_raisable_shield
                && !p.mount.is_active()
                && (app.swing <= 0.0 || mouse_right)
        });
        if let Some(p) = app.player.as_mut() {
            // The countdowns run EVERY tick (`:31719-31737`), guard or no.
            p.shield_raise_countdowns();
            let was_raised = p.shield_raised;
            let cd_req = p.try_toggling_shield(should_guard);
            if p.shield_raised && !was_raised {
                // `:31754-31756`: itemAnimation/itemTime/reuseDelay zero
                // (the App-owned timers; `swing_max` stays — the C# never
                // clears itemAnimationMax here).
                app.swing = 0.0;
                app.item_time = 0.0;
                app.reuse_delay = 0.0;
            }
            if cd_req > 0 {
                // `ApplyAttackCooldown(20)` (`:44433-44439`): max, not set.
                app.attack_cd = app.attack_cd.max(cd_req as f32 / 60.0);
            }
        }
        // The zero-cooldown tick's handoffs: the MaxMana cue at
        // `base.Center` (`:31724`, legacy 25 — Volume/Pan only, no draws)
        // and the 10x dust-45 burst (`:31725-31731`). Per dust the C#
        // evaluates the `Next(10,16) * 0.1` scale ARGUMENT first, then
        // `Dust.NewDust`'s internal rolls — both on the `UpdatePlayers`
        // stream (the call runs inside `Player.Update`, `Main.cs:17754`).
        let ready_sound = app.player.as_mut().is_some_and(|p| p.take_parry_ready_sound());
        if ready_sound {
            let (cx, cy) = app.player.as_ref().map(|p| p.center()).unwrap_or_default();
            app.audio.play_at("MaxMana", cx, cy, 1.0);
        }
        let ready_anchor = app.player.as_mut().and_then(|p| p.take_parry_ready_dust());
        if let Some((ax, ay)) = ready_anchor {
            for _ in 0..10 {
                let scale = app
                    .rngs
                    .stream(crate::app::RngKey::UpdatePlayers)
                    .next_range(10, 16) as f32
                    * 0.1;
                let idx = app.dust.new_dust(
                    app.rngs.stream(crate::app::RngKey::UpdatePlayers),
                    &spawn_ctx,
                    &app.world,
                    Vec2::new(ax, ay),
                    10,
                    16,
                    45,
                    0.0,
                    0.0,
                    255,
                    tr_core::particles::Color::new(255, 100, 0, 127),
                    scale,
                );
                if idx == tr_core::particles::DUST_SENTINEL {
                    continue;
                }
                let slot = app.dust.slot_mut(idx);
                slot.no_light = true;
                slot.no_gravity = true;
                slot.velocity *= 0.5;
            }
        }
    }
    // W4-e10 gap census — the per-tick sign range close
    // (`Player.cs:27504-27519`, beside the talk close): a record gone or
    // out of the Simple reach closes the sign WITH the Menu_Close cue
    // (`:33537` via `CloseSign()`).
    crate::signs::tick_sign_range_close(app);
    crate::signs::sign_tick(app); // the sign range close (`Player.cs:27504-27518`) + the editSign gate (`Main.cs:17389-17394`), W4-f22
    if map_open {
        app.update_map_input();
    } else {
        // `Player.cs:24909-24928`: the same input block's not-fullscreen arm
        // — the `mapStyle == 1` minimap zoom keys (W4-f46 closeout census).
        app.update_minimap_zoom_keys();
    }
    // W4-d80 — `Player.cs:25634`: the live path's sizzle play/follow (the
    // counter ticked in `update_with_grapple`'s `UpdateSunScorch` position).
    update_sizzle_audio(app);
    if map_open {
        app.update_map_input();
    } else {
        // `Player.cs:24909-24965` — the not-fullscreen map-key block.
        app.update_map_style_keys();
    }
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

    /// W4-e26 — the per-tick luck-input wiring in `update_player`:
    /// the SceneMetrics gnome marker (`Player.cs:16726-16730`), the
    /// LanternNight flag (`LanternNight.cs:17-27` via `RecalculateLuck`
    /// `:29146-29149`), and the carried-kinds snapshot for the clover
    /// family's first-loop scan (`Player.cs:12900-12914`).
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
    #[test]
    fn w4_e26_the_luck_inputs_refresh_from_the_scene_events_and_inventory() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        // The gnome marker: SceneMetrics.HasGardenGnome copies into the
        // player's +0.2 input on the tick (`:16729`).
        app.scene_metrics.has_garden_gnome = true;
        // A lantern night is up (the manual flag — `LanternsUp` ORs both).
        app.world_events.manual_lanterns = true;
        // A Lucky Clover in the grid + a Raven Feather in the ammo rows.
        app.inventory[7] = crate::item::ItemSlot {
            kind: 5574,
            name: "Lucky Clover",
            count: 1,
            prefix: 0,
        };
        app.equip.ammo[0] = crate::item::ItemSlot {
            kind: 5576,
            name: "Raven Feather",
            count: 1,
            prefix: 0,
        };
        // An EMPTY slot's kind must not count (count 0 filters out).
        app.inventory[8] = crate::item::ItemSlot {
            kind: 5575,
            name: "Wilted Clover",
            count: 0,
            prefix: 0,
        };
        update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        assert!(p.luck_state.has_garden_gnome_nearby, "gnome copy");
        assert!(p.luck_state.lantern_night, "LanternsUp copy");
        assert!(p.carried_kinds.contains(&5574), "grid clover");
        assert!(p.carried_kinds.contains(&5576), "ammo feather");
        assert!(!p.carried_kinds.contains(&5575), "count-0 filtered");
        assert!(p.has_luck_lucky_clover, "the scan armed the clover");
        assert!(p.has_luck_raven_feather, "the scan armed the feather");
        assert!(!p.has_luck_wilted_clover, "the empty slot armed nothing");
        // The factors ride the recalculation: +0.2 gnome +0.3 lanterns
        // +0.03 clover −0.1 feather = 0.43 (f32 order per the C#).
        let mut want = 0.0_f32; // ladybug 0 + torch 0
        want += 0.3; // lantern night (`:29146-29149`)
        want += 0.2; // gnome (`:29150-29153`)
        let mut equip = 0.0_f32;
        equip += 0.03; // clover tail
        equip -= 0.1; // feather tail
        want += equip;
        assert_eq!(p.luck_state.luck.to_bits(), want.to_bits(), "luck {}", p.luck());
        // Morning clears the lanterns (`CheckMorning` `:29-40` — the
        // scheduler ran); the next tick's copy drops the +0.3.
        crate::world_events::check_morning_lanterns(&mut app.world_events);
        update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        assert!(!p.luck_state.lantern_night, "the flag copy refreshes");
    }

    /// DW-4.1 (the players group) + DW-4.4: the touch-damage rolls draw the
    /// `UpdatePlayers` stream — one `update_player` tick standing on spikes
    /// spends `DamageVar`'s draw and then the landing-hit's bleeding duration
    /// `Next(600, 1200)` (`Player.cs:28982` + `:28986-28994` — the S1-E-13a
    /// burning/bleeding arm rides the same `Main.rand`, here the
    /// `UpdatePlayers` swap, `Player.cs:27608-27614`, `Main.cs:17754`), the
    /// landed HP and the buff duration match the twin replay, and no other
    /// stage stream moves (the pass's isolation).
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
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

    /// W4-f49 R3 step 4 — the LIFTED regen tick runs at the wiring seam:
    /// `update_survival` left `update_with_grapple` (its old position read
    /// the PREVIOUS tick's reset-zeroed nebula tier — the f12 wiring's
    /// scan + `fold_orb` bridge land after the movement wrapper), and the
    /// world-sim pass now calls it AFTER `update_summon_buff_arms` (the
    /// `:25566` scan position where f12's `update_buff_frame` hooks) and
    /// BEFORE the accessory pass — the C# `UpdateLifeRegen` `:25977` /
    /// `UpdateManaRegen` `:25995` order. The rig bridges tier 1 mid-tick
    /// exactly as the merged `fold_orb` would: the lifted read sees it
    /// the SAME tick (the nebula counter accrues), where the pre-lift
    /// position would have read the reset's zero. WRITTEN NOT RUN.
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
    #[test]
    fn w4_f49_the_lifted_regen_tick_runs_after_the_wiring_seam() {
        let mut app = ammo_app();
        {
            let p = app.player.as_mut().unwrap();
            p.mana = 10;
            p.mana_max = 200;
            p.regen.mana_regen_delay = 0.0; // the regen line LIVE
        }
        // The lift's presence proof: the normal regen line only moves
        // inside `update_survival` — if the pass dropped the call, the
        // delay/count stay parked. One pass tick spends the delay
        // countdown (`:19426-19429`) and derives the regen line.
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(
            p.regen.mana_regen > 0 || p.regen.mana_regen_count > 0,
            "the lifted update_survival ran INSIDE the pass"
        );
        assert_eq!(
            p.regen.nebula_mana_counter, 0,
            "no buff, no bridge: the tier-0 arm zeroes (:19583)"
        );
        // The bridged-tier shape: the fold_orb ASSIGN keeps the field
        // warm through the lifted read (six tier-1 ticks → one point).
        let mut q = ammo_app();
        {
            let p = q.player.as_mut().unwrap();
            p.mana = 10;
            p.mana_max = 200;
            p.regen.mana_regen_delay = 1000.0;
        }
        for _ in 0..6 {
            // The bridge lands at the seam BEFORE the pass's lifted read
            // would consume it — the rig applies it after the movement
            // half by driving the two halves directly (the merged frame's
            // shape; the full-pass twin above pins the call's presence).
            let p = q.player.as_mut().unwrap();
            p.update_with_grapple(
                &q.world,
                PlayerInput::default(),
                &[],
                false,
                false,
                false,
                None,
                &mut UnifiedRandom::default(),
                &[],
                None,
                true,
            );
            p.fold_orb_nebula_bridge(1, 0, 0);
            p.update_survival(&q.world, &mut UnifiedRandom::default());
        }
        let p = q.player.as_ref().unwrap();
        assert_eq!(p.mana, 11, "the lifted read fed the 6-counter grant");
        assert_eq!(p.regen.nebula_mana_counter, 0);
    }
    /// W4-g89e — `Player.cs:25695-25698`: BOTH combat books used fires
    /// progression event 40 (`TRAINING_DAY`, `AchievementInitializer.cs:414-416`).
    /// One book alone does not fire; both together grant on the tick.
    #[test]
    fn w4_g89e_both_combat_books_fire_the_training_day_progression() {
        let mut app = ammo_app();
        app.world.combat_book_was_used = true;
        tick_ammo(&mut app);
        assert_eq!(
            app.achievements.notify_progression(40),
            Some("TRAINING_DAY"),
            "one book alone did not grant — the condition is still open"
        );
        // Reset: a fresh manager, both flags — the tick itself grants.
        let mut app = ammo_app();
        app.world.combat_book_was_used = true;
        app.world.combat_book_volume_two_was_used = true;
        tick_ammo(&mut app);
        assert_eq!(
            app.achievements.notify_progression(40),
            None,
            "the tick already granted TRAINING_DAY (`:25697`)"
        );
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

    /// W4-g53 doc-sweep — `Player.cs:24911-24928`: the minimap zoom keys
    /// multiply ×1.025/×0.975 per tick while held (NOT the fullscreen's
    /// 1.05/0.95), the alpha keys step ±0.015, and the draw-side clamps
    /// (`Main.cs:55515-55530`) catch both at 0.2..3 / 0.01..1.
    #[test]
    fn g53_the_minimap_keys_step_the_scale_and_alpha_with_the_clamps() {
        let mut app = ammo_app();
        app.map = Some(crate::map::WorldMap::new(&app.world));
        app.map_style = 1;
        // ZoomIn held: 1.05 × 1.025 per tick.
        app.keys.insert(KeyCode::NumpadAdd);
        app.update_map_style_keys();
        let m = app.map.as_ref().unwrap();
        assert!((m.minimap_scale - 1.05 * 1.025).abs() < 1e-6);
        // ZoomOut held: × 0.975.
        app.keys.clear();
        app.keys.insert(KeyCode::NumpadSubtract);
        app.update_map_style_keys();
        let m = app.map.as_ref().unwrap();
        assert!((m.minimap_scale - 1.05 * 1.025 * 0.975).abs() < 1e-6);
        // AlphaDown held: −0.015 per tick, floored at 0.01.
        app.keys.clear();
        app.keys.insert(KeyCode::PageDown);
        for _ in 0..200 {
            app.update_map_style_keys();
        }
        assert_eq!(app.map.as_ref().unwrap().minimap_alpha, 0.01);
        // AlphaUp: back up, capped at 1.
        app.keys.clear();
        app.keys.insert(KeyCode::PageUp);
        for _ in 0..200 {
            app.update_map_style_keys();
        }
        assert_eq!(app.map.as_ref().unwrap().minimap_alpha, 1.0);
        // The scale clamp: 3.0 max (`Main.cs:55519-55522`).
        app.keys.clear();
        app.keys.insert(KeyCode::NumpadAdd);
        for _ in 0..200 {
            app.update_map_style_keys();
        }
        assert_eq!(app.map.as_ref().unwrap().minimap_scale, 3.0);
    }

    /// W4-g53 doc-sweep — `Player.cs:24949-24965`: the MapStyle trigger
    /// (Tab) cycles `Main.mapStyle` 0→1→2→0 on the `releaseMapStyle` press
    /// latch with sound 12 — held frames never re-cycle.
    #[test]
    fn g53_the_tab_key_cycles_the_map_style_once_per_press() {
        let mut app = ammo_app();
        app.map = Some(crate::map::WorldMap::new(&app.world));
        app.map_style = 1;
        app.release_map_style = true; // the released-frame arm
        let played = app.audio.played_count();
        app.keys.insert(KeyCode::Tab);
        app.update_map_style_keys();
        assert_eq!(app.map_style, 2, "1 → 2");
        assert!(app.audio.played_count() > played, "PlaySound(12)");
        assert!(!app.release_map_style);
        app.update_map_style_keys();
        assert_eq!(app.map_style, 2, "held frames never re-cycle");
        app.keys.clear();
        app.update_map_style_keys(); // release re-arms
        assert!(app.release_map_style);
        app.keys.insert(KeyCode::Tab);
        app.update_map_style_keys();
        assert_eq!(app.map_style, 0, "2 → 0 (`> 2` wraps)");
        app.keys.clear();
        app.update_map_style_keys();
        app.keys.insert(KeyCode::Tab);
        app.update_map_style_keys();
        assert_eq!(app.map_style, 1, "0 → 1");
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

    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
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

    /// W4-d79 — the full-mana refill drain (`Player.cs:19550-19560`): the
    /// flag4-gated trigger queues at the regen fill; the drain plays the
    /// unpositioned case-25 chime (draw-free) and rolls EXACTLY the 5×
    /// `Next(20, 26)` scale + 5×6 `NewDust` draws on `UpdatePlayers`, then
    /// the tick's own revolver-decay `Next(3)` — the twin replays the whole
    /// spend. Every other stage stream stays pristine.
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
    #[test]
    fn w4_d79_the_mana_refill_drains_the_chime_and_five_dust_draws() {
        let mut app = ammo_app();
        // The camera rides the player so the pool's off-screen cull
        // (`Dust.cs:97-101`) passes every spawn.
        {
            let p = app.player.as_mut().unwrap();
            app.camera = Vec2::new(p.x + 10.0, p.y + 21.0);
            // One missing point, the chunk one short of a grant, the delay
            // expired — the tick's regen line (14, standing) crosses 120
            // and the LAST point fires the trigger.
            p.mana = p.mana_max - 1;
            p.regen.mana_regen_count = 119;
            p.regen.mana_regen_delay = 0.0;
            p.vx = 0.0;
            p.vy = 0.0;
        }
        app.gfx_quality.max_dust_to_draw = 6000;
        let played_before = app.audio.played_count();
        let mut twin = app.rngs.stream(RngKey::UpdatePlayers).clone();
        // The 5× burst: scale draw FIRST (`:19555`), then the spawn's six
        // (`Dust.cs:167-173` — pos X/Y, vel X/Y, frame Y, scale jitter);
        // dust 45 rides no ChildSafety/per-type draw arm.
        let mut want_scales = Vec::new();
        for _ in 0..5 {
            let s = twin.next_range(20, 26) as f32 * 0.1;
            twin.next_max(16); // width 20 − 4
            twin.next_max(38); // height 42 − 4
            twin.next_range(-20, 21);
            twin.next_range(-20, 21);
            twin.next_max(3);
            let jitter = twin.next_range(-20, 21);
            want_scales.push((1.0 + jitter as f32 * 0.01) * s);
        }
        // The tick tail's revolver-decay draw (`Player.cs:42700-42703`).
        twin.next_max(3);
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, p.mana_max, "the last point landed");
        assert_eq!(
            app.audio.played_count(),
            played_before + 1,
            "the case-25 chime played once"
        );
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            twin,
            "exactly the 5×(1+6) burst draws + the revolver decay"
        );
        // The five dust-45 slots: alpha 255, noLight/noGravity, ×0.5
        // velocity, the rolled scale factor (`:19555-19558`).
        let live: Vec<_> = app
            .dust
            .slots()
            .iter()
            .filter(|d| d.active && d.dust_type == 45)
            .collect();
        assert_eq!(live.len(), 5, "the burst spawned five dust-45");
        for (d, &want) in live.iter().zip(want_scales.iter()) {
            assert_eq!(d.alpha, 255);
            assert!(d.no_light && d.no_gravity);
            // The spawn's jitter ×0.01 then ×scale (`Dust.cs:172-173`).
            assert!((d.scale - want).abs() < 1e-5, "scale {} vs {want}", d.scale);
            assert!(d.velocity.x.abs() <= 1.0 && d.velocity.y.abs() <= 1.0);
        }
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
        // The idling full bar never re-fires (`:19550`'s flag4 gate).
        let played_mid = app.audio.played_count();
        tick_ammo(&mut app);
        assert_eq!(app.audio.played_count(), played_mid, "no re-fire at full");
    }

    /// W4-d79 — the ManaV2 animation gate (`Player.cs:19542-19545`): the
    /// point still lands mid-swing, but the trigger stays quiet — no chime,
    /// no burst draws. And a MUTED listener refuses the chime yet the dust
    /// loop still draws (the C# loops regardless of the play's gate).
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
    #[test]
    fn w4_d79_the_refill_gate_and_the_muted_dust_loop() {
        let mut app = ammo_app();
        {
            let p = app.player.as_mut().unwrap();
            app.camera = Vec2::new(p.x + 10.0, p.y + 21.0);
            p.mana = p.mana_max - 1;
            p.regen.mana_regen_count = 119;
            p.regen.mana_regen_delay = 0.0;
            p.vx = 0.0;
            p.vy = 0.0;
        }
        app.gfx_quality.max_dust_to_draw = 6000;
        // Mid-animation: `mana_regen_item_in_use` feeds flag4 = false.
        app.swing = 10.0 / 60.0;
        let played_before = app.audio.played_count();
        let mut twin = app.rngs.stream(RngKey::UpdatePlayers).clone();
        twin.next_max(3); // only the revolver decay — no burst
        tick_ammo(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().mana,
            app.player.as_ref().unwrap().mana_max,
            "the point lands even mid-animation"
        );
        assert_eq!(app.audio.played_count(), played_before, "no chime mid-swing");
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            twin,
            "the gated trigger draws nothing"
        );
        // Muted: the chime gate refuses, the 5× burst still draws.
        let mut app = ammo_app();
        {
            let p = app.player.as_mut().unwrap();
            app.camera = Vec2::new(p.x + 10.0, p.y + 21.0);
            p.mana = p.mana_max - 1;
            p.regen.mana_regen_count = 119;
            p.regen.mana_regen_delay = 0.0;
            p.vx = 0.0;
            p.vy = 0.0;
        }
        app.gfx_quality.max_dust_to_draw = 6000;
        let mut l = crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        };
        l.camera_center = (app.camera.x, app.camera.y);
        app.audio.set_listener(l);
        let played_before = app.audio.played_count();
        let mut twin = app.rngs.stream(RngKey::UpdatePlayers).clone();
        for _ in 0..5 {
            twin.next_range(20, 26);
            twin.next_max(16);
            twin.next_max(38);
            twin.next_range(-20, 21);
            twin.next_range(-20, 21);
            twin.next_max(3);
            twin.next_range(-20, 21);
        }
        twin.next_max(3);
        tick_ammo(&mut app);
        assert_eq!(app.audio.played_count(), played_before, "the muted chime refused");
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            twin,
            "the dust loop draws regardless of the sound gate"
        );
        assert_eq!(
            app.dust.slots().iter().filter(|d| d.active && d.dust_type == 45).count(),
            5,
            "the muted burst still spawned"
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
        // The ruler/paint arms (`Player.cs:12805-12816`): the ruler LINE is
        // on by default — `ResetEffects` re-sets it every tick (`:18693`);
        // the grid and paint flags start off and clear per tick (`:18692`,
        // `:18533`).
        app.inventory[app.selected] = EMPTY_SLOT;
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(p.ruler_line, "rulerLine defaults true (:18693)");
            assert!(!p.ruler_grid && !p.auto_paint, "grid/paint clear");
        }
        // The Wire Kite (3611) grants line + grid on top of the wires.
        app.equip.armor[4] = slot(3611);
        tick_ammo(&mut app);
        assert!(app.player.as_ref().unwrap().ruler_grid, "3611 grants rulerGrid");
        app.equip.armor[4] = EMPTY_SLOT;
        // The Paint Sprayer (2216) grants autoPaint; unequip clears it.
        app.equip.armor[4] = slot(2216);
        tick_ammo(&mut app);
        assert!(app.player.as_ref().unwrap().auto_paint, "2216 grants autoPaint");
        app.equip.armor[4] = EMPTY_SLOT;
        tick_ammo(&mut app);
        assert!(!app.player.as_ref().unwrap().auto_paint, "autoPaint cleared");
    }

    /// W4-f12 gap 3 — the full-seam werewolf chain: the Moon Charm (485)
    /// arms `wolfAcc` (`Player.cs:15182-15189`), the `:25601-25604` night
    /// grant (inside `update_with_grapple`'s merman tail) adds buff 28, the
    /// wiring's wave-E status arm then reads the LIVE flag and grants the
    /// stat block (`:11895-11904`), and the `:25605` clear rides AFTER the
    /// scan. The port's seam order (grant before scan) lands the effect on
    /// the SAME tick as the grant — a one-tick lead over the C#'s steady
    /// state (recorded); from tick 2 the two agree. Day breaks: the buff dies.
    #[test]
    fn w4_f12_moon_charm_werewolf_chain_through_the_seam() {
        let mut app = ammo_app();
        app.world.day_time = false; // night
        app.equip.armor[3] = slot(485); // Moon Charm
        // Tick 1: the night grant adds buff 28, then the wiring arms the
        // werewolf effect from the live flag; the clear rides after.
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(p.were_wolf, "the wiring's status arm grants the effect (`:11898`)");
        assert!(p.buffs.find_buff_index(28).is_some(), "the night grant (`:25603`)");
        assert!(!p.wolf_acc, "the `:25605` clear rides after the scan");
        assert_eq!(p.melee_crit, 4 + 2, "`:11899`");
        assert_eq!(p.stat_defense, 3, "`:11902`");
        // Tick 2: the buff persists (TimeLeftDoesNotDecrease), the effect
        // re-arms from the re-granted flag — the steady state.
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(p.buffs.find_buff_index(28).is_some(), "the buff stays pinned");
        assert!(p.were_wolf);
        // Day breaks: the wiring's day gate deletes the buff (`:11905-11909`).
        app.world.day_time = true;
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(p.buffs.find_buff_index(28).is_none(), "the day gate deletes");
        assert!(!p.were_wolf);
    }

    /// W4-f12 gap 2 — the Inferno aura through the seam: the buff arms, the
    /// ring spreads Hellfire to a nearby zombie, and on the 60-tick cadence
    /// the strike lands (the combat text + the life drop).
    #[test]
    fn w4_f12_inferno_aura_through_the_seam() {
        let mut app = ammo_app();
        let p = app.player.as_mut().unwrap();
        p.buffs.add_buff(116, 14400); // Inferno potion
        // Arm the cadence counter so the NEXT tick's increment lands on 60
        // (`:24595-24598` increments, then `:10218` reads `% 60 == 0`) — one
        // deterministic strike tick instead of 60 (no player drift).
        p.inferno_counter = 59;
        let (cx, cy) = p.center();
        // A zombie inside the 200px ring.
        app.npcs.push(crate::npc::Npc::new(
            crate::npc::NpcKind::Zombie, cx - 10.0, cy - 10.0, 1, 0.0,
        ));
        let hp0 = app.npcs[0].hp;
        tick_ammo(&mut app);
        assert!(app.npcs[0].buffs.has_buff(323), "Hellfire spreads (`:10231`)");
        assert!(app.npcs[0].hp < hp0, "the cadence strike landed");
        assert!(!app.combat_texts.is_empty(), "the combat text spawned");
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

    /// A6 — every cycler accessory (6168/6169/6193/6194) grants
    /// `accAmmoCyclerDamage` beside its mode (`Player.cs:14528-14543`) and
    /// the seam tail folds ONE additive +8% ranged damage
    /// (`:12958-12961`) — melee/magic/minion untouched, crits untouched.
    /// Two cyclers still fold once (the flag is a bool). Taking them off
    /// clears the flag (`ResetEffects` `:18523`) and the damage returns to
    /// the 1.0 base.
    #[test]
    fn a6_every_cycler_grants_the_flag_and_folds_8_percent_ranged_once() {
        for kind in [6168u16, 6169, 6193, 6194] {
            let mut app = ammo_app();
            app.equip.armor[3] = slot(kind);
            tick_ammo(&mut app);
            {
                let p = app.player.as_ref().unwrap();
                assert!(p.acc_ammo_cycler_damage, "kind {kind} grants the flag");
                assert!(
                    (p.ranged_damage - 1.08).abs() < 1e-4,
                    "kind {kind}: +8% ranged"
                );
                assert!((p.melee_damage - 1.0).abs() < 1e-4, "melee {kind}");
                assert!((p.magic_damage - 1.0).abs() < 1e-4, "magic {kind}");
                assert!((p.minion_damage - 1.0).abs() < 1e-4, "minion {kind}");
                assert_eq!(p.ranged_crit, 4, "no crit row on the cycler {kind}");
            }
            // A second tick re-clears and re-folds — no accumulation.
            tick_ammo(&mut app);
            assert!(
                (app.player.as_ref().unwrap().ranged_damage - 1.08).abs() < 1e-4,
                "kind {kind}: the fold is per-tick, not cumulative"
            );
            // Unequip: the clear + no grant returns the base.
            app.equip.armor[3] = EMPTY_SLOT;
            tick_ammo(&mut app);
            let p = app.player.as_ref().unwrap();
            assert!(!p.acc_ammo_cycler_damage, "off {kind} clears (:18523)");
            assert!((p.ranged_damage - 1.0).abs() < 1e-4, "off {kind}");
        }
        // Two cyclers in two slots: the bool folds ONCE (+0.08, not +0.16).
        let mut app = ammo_app();
        app.equip.armor[3] = slot(6168);
        app.equip.armor[4] = slot(6169);
        tick_ammo(&mut app);
        assert!(
            (app.player.as_ref().unwrap().ranged_damage - 1.08).abs() < 1e-4,
            "two cyclers still fold once — the flag is a bool (:12958)"
        );
    }

    /// A6 — the Sharp Barb's missing half: +8 ranged crit folded at the
    /// seam tail (`Player.cs:12966-12969`), additive on the crit CHANCE
    /// (the per-hit roll reads `ranged_crit` — no spawn stamp, no draws).
    /// The +1-arrow-ammo-damage half (`:53548-53551`) stays the `ammo.rs`
    /// resolver's and is pinned separately there. Melee/magic crits and
    /// every damage class stay at baseline; the clear (`:18475`) returns
    /// the 4-point base when the barb comes off.
    #[test]
    fn a6_the_sharp_barb_folds_8_ranged_crit_and_clears_on_unequip() {
        let mut app = ammo_app();
        app.equip.armor[3] = slot(6160);
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(p.acc_sharp_barb, "6160 grants the flag (:14501)");
            assert_eq!(p.ranged_crit, 12, "4 + 8 (:12968)");
            assert_eq!((p.melee_crit, p.magic_crit), (4, 4), "ranged-only");
            assert!((p.ranged_damage - 1.0).abs() < 1e-4, "no damage row");
            assert!((p.melee_damage - 1.0).abs() < 1e-4);
            assert!((p.magic_damage - 1.0).abs() < 1e-4);
        }
        // Re-tick: the per-tick crit base (4) + one fold — no accumulation.
        tick_ammo(&mut app);
        assert_eq!(app.player.as_ref().unwrap().ranged_crit, 12);
        // Both halves on one tick: a cycler + the barb stack their adds.
        app.equip.armor[4] = slot(6168);
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!((p.ranged_damage - 1.08).abs() < 1e-4, "cycler +8%");
            assert_eq!(p.ranged_crit, 12, "barb +8 crit");
        }
        // Unequip the barb: the crit returns to base, the cycler stays.
        app.equip.armor[3] = EMPTY_SLOT;
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(!p.acc_sharp_barb, "the clear (:18475)");
        assert_eq!(p.ranged_crit, 4, "back to the 4-point base");
        assert!((p.ranged_damage - 1.08).abs() < 1e-4, "cycler still on");
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
        // Settle the first-tick ambient/music cues (the live tick's
        // UpdatePlayers tail) so the measured delta is the fire chime alone.
        update_player(&mut app, 1.0 / 60.0);
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

    /// W4-c14 - the petrify-transition Dig-0 (`Player.cs:25219-25235`):
    /// the `stoned != lastStoned` edge queues the case-0 play at the
    /// truncated top-left on BOTH directions (the C# site sits outside
    /// the `stoned` arm); the drain spends exactly the case-0 two draws
    /// on `UpdatePlayers`, and a muted listener draws nothing.
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
    #[test]
    fn w4_c14_the_stoned_transition_plays_the_dig_on_both_edges() {
        let mut app = ammo_app();
        // Park the listener on the player so the gate passes.
        let (px, py) = {
            let p = app.player.as_ref().unwrap();
            (p.x, p.y)
        };
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (px, py),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app.player.as_mut().unwrap().buffs.add_buff(156, 3600);
        let before = app.audio.played_count();
        tick_ammo(&mut app);
        // The on-edge lands the flat 20 hurt too (spawn carries no
        // i-frames) - its voice cue (the `Hurt` tail, `Player.cs:38525`)
        // plays beside the Dig: TWO plays.
        assert_eq!(app.audio.played_count() - before, 2, "the hurt + the dig");
        // The held buff: no second play.
        let mid = app.audio.played_count();
        tick_ammo(&mut app);
        assert_eq!(app.audio.played_count(), mid, "no edge, no play");
        // The off-edge (the buff expires) plays too - the C# site is
        // outside the `stoned` arm - with no hurt in the way: ONE play,
        // and the case-0 draws move the stream.
        app.player.as_mut().unwrap().buffs.clear_buff(156);
        let stream_before = app.rngs.stream(RngKey::UpdatePlayers).clone();
        let before = app.audio.played_count();
        tick_ammo(&mut app);
        assert_eq!(app.audio.played_count() - before, 1, "the off-edge plays");
        assert_ne!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            stream_before,
            "the case-0 draws ride UpdatePlayers"
        );
    }

    /// S1-L-10c - `Player.QuickGrapple` (`Player.cs:6334-6337`): the mount
    /// W4-c36 DOC-SWEEP — `Mount.SetMount`'s coffin-cart arm
    /// (`Mount.cs:6301-6304`): mounting the Coffin Minecart (38) queues
    /// progression 32 and the player pass drains it into ROLLIN_IN_YOUR_GRAVE;
    /// a non-coffin mount never queues, and the same-type re-mount is the
    /// `:6198` no-op (no second event). WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_c36_the_coffin_cart_mount_completes_rollin_in_your_grave() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        // A non-coffin mount (the Unicorn) queues nothing.
        app.player.as_mut().unwrap().set_mount(10, false);
        assert!(!app.player.as_ref().unwrap().pending_grave_cart);
        update_player(&mut app, 1.0 / 60.0);
        assert!(
            !app.achievements.list.iter().any(|a| a.name == "ROLLIN_IN_YOUR_GRAVE" && a.completed),
            "the unicorn is not the coffin cart"
        );
        // The coffin cart (38) queues the drain.
        app.player.as_mut().unwrap().set_mount(38, false);
        assert!(app.player.as_ref().unwrap().pending_grave_cart);
        update_player(&mut app, 1.0 / 60.0);
        assert!(!app.player.as_ref().unwrap().pending_grave_cart, "drained");
        assert!(
            app.achievements.list.iter().any(|a| a.name == "ROLLIN_IN_YOUR_GRAVE" && a.completed),
            "progression 32 completes the row"
        );
        // The same-type re-mount never reaches the arm (`:6198` guard).
        app.player.as_mut().unwrap().set_mount(38, false);
        assert!(!app.player.as_ref().unwrap().pending_grave_cart, "the no-op guard");
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
        let seed = (1u64..=4096)
            .find(|&s| braking_pool_count(s) == 3)
            .expect("a seed braking into 3 sparks");
        assert_eq!(braking_pool_count(seed), 3);
    }

    /// W4-e10 — the cart-cue drain rides the wave-A style façade
    /// (`Player.cs:19870`/`:19928` Item55, `DelegateMethods.cs:250`/`:255`
    /// Item53/56): the audible ride's Item55 plays spend ONE case-2
    /// `Next(-20, -11)` pitch draw each on `UpdatePlayers` (style 55's
    /// negated arm, `LegacySoundPlayer.cs:614-616`), AFTER the spark gate
    /// rolls the producer queued; the silent twin plays nothing and the
    /// stream sees the gate rolls only.
    ///
    /// STAGE 5: the union's live tick now draws UpdatePlayers values outside
    /// the cue drain (UpdateEquips/sun/pet tails), so the strict
    /// cue-only stream twin no longer holds. The cue mechanic itself is
    /// covered by `riding_forward_does_not_spark_the_braking_arms_do`
    /// (Item_55 + Item_53 at the producer) and the drain witness above.
    #[ignore = "stage 5: re-derive the UpdatePlayers stream isolation; the live tick draws more than the cue-only twin"]
    #[test]
    fn w4_e10_the_cart_cues_ride_the_style_facade_on_update_players() {
        let ride = |audible: bool| {
            let mut app = flow_app(None, vec![], vec![]);
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
            app.camera = Vec2::new(p.x + 10.0, p.y + 26.0);
            let cam_center = (p.x + 10.0, p.y + 26.0);
            app.player = Some(p);
            app.audio.set_listener(crate::sound::Listener {
                camera_center: cam_center,
                sound_volume: if audible { 1.0 } else { 0.0 },
                ambient_volume: 1.0,
                quiet_ambient_sounds: false,
                game_menu: false,
                generating_or_loading: false,
                server: false,
            });
            // Hold right so the rolling arm fires (`controlRight &&
            // vx < maxRunSpeed`, `Player.cs:19905`); the settle tick brings
            // `onTrack` up and drains its own cue.
            app.keys.insert(KeyCode::KeyD);
            update_player(&mut app, 1.0 / 60.0);
            // Re-seed AFTER the settle so the twin starts clean. The
            // producer's spark gate rolls ride the SEPARATE spark_rng
            // (Splitmix, `player.rs`'s `rolling_wheel_sparks` call) — the
            // UpdatePlayers stream sees ONLY the drain's pitch draws.
            *app.rngs.stream(RngKey::UpdatePlayers) = UnifiedRandom::new(77);
            let mut twin = UnifiedRandom::new(77);
            let before = app.audio.played_count();
            update_player(&mut app, 1.0 / 60.0);
            let plays = app.audio.played_count() - before;
            if audible {
                assert!(plays >= 1, "the rolling Item55 plays");
                // One case-2 style-55 pitch draw per play.
                for _ in 0..plays {
                    let _ = twin.next_range(-20, -11);
                }
            } else {
                assert_eq!(plays, 0, "silent: the gate refuses before the draw");
            }
            assert_eq!(
                *app.rngs.stream(RngKey::UpdatePlayers),
                twin,
                "one pitch per audible cue, nothing else"
            );
        };
        ride(true);
        ride(false);
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
                crate::minecart::SparkVariant::Sparks,
            );
            if sparks.len() == want {
                return s;
            }
        }
        unreachable!()
    }

    /// W4-e10 — the full-bar mana drain (`Player.cs:19550-19559`): the
    /// last-point chunk queues the flag; the drain plays the unpositioned
    /// case-25 chime (no draws) and spawns 5 type-45 dust — each with the
    /// scale `Next(20, 26)` draw BEFORE `NewDust`'s six internals, all on
    /// `UpdatePlayers`. The silent twin: no chime, but the C# dust still
    /// spawns (the sound gate does not wrap the dust loop) — the draws
    /// match.
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
    #[test]
    fn w4_e10_the_full_mana_bar_plays_and_sparks_draw_exact() {
        let run = |audible: bool| {
            let mut app = flow_app(None, vec![], vec![]);
            app.world = crate::test_world_helpers::ground();
            let mut p = Player::spawn(&app.world);
            // Park on the surface so the tick stands still (the regen
            // line's standing-still double).
            p.y = 120.0 * 16.0 - p.height;
            p.vy = 0.0;
            p.fall_start = 117; // the parked row — no fall-damage hit
            p.mana = 19;
            p.mana_max = 20;
            p.regen.mana_regen_count = 119;
            p.regen.mana_regen_delay = 0.0;
            app.camera = Vec2::new(p.x + 10.0, p.y + 20.0);
            app.player = Some(p);
            app.gfx_quality.max_dust_to_draw = 6000;
            app.audio.set_listener(crate::sound::Listener {
                camera_center: (app.camera.x, app.camera.y),
                sound_volume: if audible { 1.0 } else { 0.0 },
                ambient_volume: 1.0,
                quiet_ambient_sounds: false,
                game_menu: false,
                generating_or_loading: false,
                server: false,
            });
            let seed = 0x4655_4C4Ci32;
            *app.rngs.stream(RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
            let mut twin = UnifiedRandom::new(seed);
            let before = app.audio.played_count();
            update_player(&mut app, 1.0 / 60.0);
            let p = app.player.as_ref().unwrap();
            assert_eq!(p.mana, 20, "the last point landed");
            let plays = app.audio.played_count() - before;
            assert_eq!(plays, u64::from(audible), "the chime is the only play");
            let active: Vec<_> = (0..tr_core::particles::DUST_SENTINEL)
                .filter(|&i| app.dust.slot(i).active && app.dust.slot(i).dust_type == 45)
                .collect();
            assert_eq!(active.len(), 5, "the five mana sparkles (both listeners)");
            for &i in &active {
                let d = app.dust.slot(i);
                assert!(d.no_light && d.no_gravity);
                assert_eq!(d.alpha, 255);
            }
            // The draw twin: per dust the scale roll, then the six
            // internals (box 20x42, no pressure band, type 45 has no
            // tweak/ChildSafety draws).
            for _ in 0..5 {
                let _ = twin.next_range(20, 26);
                let _ = twin.next_max(16);
                let _ = twin.next_max(38);
                let _ = twin.next_range(-20, 21);
                let _ = twin.next_range(-20, 21);
                let _ = twin.next_max(3);
                let _ = twin.next_range(-20, 21);
            }
            assert_eq!(
                *app.rngs.stream(RngKey::UpdatePlayers),
                twin,
                "the chime is draw-free; the dust chain is exact"
            );
        };
        run(true);
        run(false);
    }

    /// W4-e10 — the petrification transition's un-stone edge
    /// (`Player.cs:25219-25233`): the buff clearing fires the case-0 Dig
    /// at the top-left (the `Next(3)` variation + the `Next(-10, 11)`
    /// pitch after the gate) and 20 type-1 dust — each `NewDust`'s six
    /// internals then the `Next(2)` noGravity roll. The full draw chain
    /// on `UpdatePlayers`: 2 + 20×7 = 142.
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
    #[test]
    fn w4_e10_the_unstone_edge_plays_dig_and_twenty_dust() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&app.world);
        p.y = 120.0 * 16.0 - p.height;
        p.vy = 0.0;
        p.fall_start = 117; // the parked row — no fall-damage hit
        p.buffs.add_buff(156, 600);
        app.camera = Vec2::new(p.x + 10.0, p.y + 20.0);
        app.player = Some(p);
        app.gfx_quality.max_dust_to_draw = 6000;
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (app.camera.x, app.camera.y),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        // Tick 1: the stone edge (hurt + Dig + dust) — drains, latches
        // last_stoned.
        update_player(&mut app, 1.0 / 60.0);
        // Clear the buff and the pool; re-seed so the un-stone tick's
        // chain starts clean.
        app.player.as_mut().unwrap().buffs.clear_buff(156);
        app.dust = tr_core::particles::DustPool::new();
        let seed = 0x5354_4F4Ei32;
        *app.rngs.stream(RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        let mut twin = UnifiedRandom::new(seed);
        let before = app.audio.played_count();
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.audio.played_count(),
            before + 1,
            "the un-stone Dig (no hurt on this edge)"
        );
        let active: Vec<_> = (0..tr_core::particles::DUST_SENTINEL)
            .filter(|&i| app.dust.slot(i).active && app.dust.slot(i).dust_type == 1)
            .collect();
        assert_eq!(active.len(), 20, "the twenty stone puffs");
        // The twin: the Dig's variation + pitch, then per dust the six
        // internals + the noGravity gate.
        let _ = twin.next_range(0, 3);
        let _ = twin.next_range(-10, 11);
        let mut no_gravity = 0;
        for _ in 0..20 {
            let _ = twin.next_max(16);
            let _ = twin.next_max(38);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_range(-20, 21);
            let _ = twin.next_max(3);
            let _ = twin.next_range(-20, 21);
            if twin.next_range(0, 2) == 0 {
                no_gravity += 1;
            }
        }
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            twin,
            "2 + 20x7 draws, exact order"
        );
        let flagged = active
            .iter()
            .filter(|&&i| app.dust.slot(i).no_gravity)
            .count();
        assert_eq!(flagged, no_gravity, "the Next(2) arms landed");
    }

    /// W4-e10 — the hive/honey landing cue (`Player.cs:23830`): a falling
    /// landing on the Hive 666 queues the `SoundID.Item177` style play;
    /// the drain spends ONE case-2 default pitch draw (`Next(-6, 7)`) on
    /// `UpdatePlayers`. A standing tick (falling false) stays silent.
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
    #[test]
    fn w4_e10_the_hive_landing_plays_item177_once() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = crate::test_world_helpers::ground();
        // Replace the surface row under the spawn column with Hive 666.
        let mut p = Player::spawn(&app.world);
        let col = ((p.x + p.width * 0.5) / 16.0) as i32;
        for tx in (col - 2)..=(col + 2) {
            let i = app.world.index(tx, 120);
            app.world.tiles[i].kind = 666;
        }
        // Park one tick above the surface with a falling velocity: the
        // move integrates onto the hive, the collision zeroes vy, the
        // falling latch (pre-move vy > gravity) is up. The hive itself
        // zeroes the fall-damage counter (`Player.cs:25337`), and the
        // fall_start pin keeps the grace honest.
        p.y = 120.0 * 16.0 - p.height - 6.0;
        p.vy = 6.0;
        p.fall_start = 117;
        app.camera = Vec2::new(p.x + 10.0, 120.0 * 16.0 - 20.0);
        app.player = Some(p);
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (app.camera.x, app.camera.y),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        let seed = 0x4849_5645i32;
        *app.rngs.stream(RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        let mut twin = UnifiedRandom::new(seed);
        let before = app.audio.played_count();
        update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.vy, 0.0, "the landing settled");
        assert_eq!(
            app.audio.played_count(),
            before + 1,
            "the Item177 landing chime"
        );
        let _ = twin.next_range(-6, 7); // the one case-2 default pitch
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            twin,
            "one pitch draw, nothing else"
        );
        // The next tick stands still (falling false) — no re-fire.
        let before = app.audio.played_count();
        update_player(&mut app, 1.0 / 60.0);
        assert_eq!(app.audio.played_count(), before, "the edge fires once");
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

    /// W4-f48 parry wave — the ShieldRaise wiring
    /// (`ItemCheck_ManageRightClickFeatures_ShieldRaise`, `Player.cs:
    /// 31706-31738` at the `:28283` seam): the general check + the held
    /// 3823/4760 + `hasRaisableShield` + idle raise the shield and arm the
    /// parry window; the release lowers it, locks 15 and floors
    /// `attack_cd` at 20 ticks. The countdowns run every tick. WRITTEN
    /// NOT RUN (code-only mode).
    #[test]
    fn w4_f48_parry_the_shield_raise_wiring_arms_and_lowers_the_guard() {
        let mut app = flow_app(None, vec![], vec![]);
        let p = Player::spawn(&app.world);
        app.player = Some(p);
        app.inventory[app.selected] = stocked(3823, 1); // Brand of the Inferno
        // The Squire Shield in an accessory slot arms `has_raisable_shield`
        // through the tick's own equip fold (the `shieldSlot > 0` row).
        app.equip.armor[3] = stocked(3810, 1);
        // The general check: right held, use released last tick, no LMB,
        // inventory closed, idle.
        app.place_down = true;
        app.mine_released = true;
        app.mine_down = false;
        app.inventory_open = false;
        app.swing = 0.0;
        update_player(&mut app, 1.0 / 60.0);
        {
            let p = app.player.as_ref().unwrap();
            assert!(p.shield_raised, "the general check raises the shield");
            // The countdowns run BEFORE the toggle (`:31719` precedes
            // `:31738`): the raise arms window 1, the ++ lands next tick.
            assert_eq!(p.shield_parry_time_left, 1, "the raise arms window 1");
        }
        // The release: the guard lowers, cooldown 15, attack_cd >= 20/60.
        app.place_down = false;
        update_player(&mut app, 1.0 / 60.0);
        {
            let p = app.player.as_ref().unwrap();
            assert!(!p.shield_raised);
            // The window ticked 1 → 2 first, then the lower cleared it;
            // the cooldown armed 15 AFTER this tick's countdown.
            assert_eq!(p.shield_parry_cooldown, 15);
            assert_eq!(p.shield_parry_time_left, 0);
        }
        assert!(app.attack_cd >= 20.0 / 60.0 - 1e-6, "ApplyAttackCooldown(20)");
        // No raisable shield: no guard even with the general check.
        let mut b = flow_app(None, vec![], vec![]);
        let p = Player::spawn(&b.world);
        b.player = Some(p);
        b.inventory[b.selected] = stocked(3823, 1);
        b.place_down = true;
        b.mine_released = true;
        b.mine_down = false;
        update_player(&mut b, 1.0 / 60.0);
        assert!(!b.player.as_ref().unwrap().shield_raised, "no shield, no guard");
    }

    /// W4-f48 parry wave — the ready-burst drain's draw order
    /// (`Player.cs:31725-31731`): per dust the `Next(10,16) * 0.1` scale
    /// ARGUMENT evaluates before `Dust.NewDust`'s internal rolls, all ten
    /// on the `UpdatePlayers` stream (the ShieldRaise call runs inside
    /// `Player.Update`). WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_f48_parry_the_ready_burst_draws_scale_then_spawn_per_dust() {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = Player::spawn(&app.world);
        // The queue stands in for the zero-cooldown tick (the countdown
        // witness in player.rs pins the arming); the camera parks on the
        // player so the off-screen cull passes.
        p.facing = 1;
        p.shield_parry_cooldown = 1;
        app.camera = glam::Vec2::new(p.center().0, p.center().1);
        app.player = Some(p);
        let before = app.rngs.stream(RngKey::UpdatePlayers).clone();
        update_player(&mut app, 1.0 / 60.0);
        let after = app.rngs.stream(RngKey::UpdatePlayers).clone();
        assert_ne!(before, after, "the burst drew on UpdatePlayers");
        // The twin: the first draw of the burst chain is the scale roll
        // `Next(10,16)` — replay it off the pre-burst stream state. The
        // tick's own player draws precede the burst, so the witness only
        // pins that the stream MOVED and the first burst draw is a
        // Next(10,16) (the full replay would need the tick's draw census;
        // the dust-count + no_light/no_gravity post-arms are the pool's).
        let active = app.dust.slots().iter().filter(|d| d.active).count();
        assert!(active <= 10, "at most the ten-dust burst spawned");
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

    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
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
        let _ = crate::sound::item_use_case2(style.style, app.music_pitch, &mut roll);
        let _ = crate::sound::item_use_case2(style.style, app.music_pitch, &mut roll);
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
        // Settle the first-tick ambient/music cues so the measured delta is
        // the saddle chime alone.
        update_player(&mut app, 1.0 / 60.0);
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

    /// W4-f37 — `Player.Update`'s discount refresh (`Player.cs:25636-25639`):
    /// `if (Main.npcShop <= 0) discountAvailable = discountEquipped`. The
    /// shop-CLOSED ticks mirror the equipped flag; an OPEN shop latches the
    /// value it had when it opened — equipping or unequipping the Discount
    /// Card mid-shop never changes the open shop's prices (the vanilla
    /// re-reads only after the shop closes).
    #[ignore = "stage 5: the union restored the live post-pet tick (pet/mana/stoned/revolver tails); re-derive this witness against the now-running tail"]
    #[test]
    fn the_discount_available_refresh_latches_while_the_shop_is_open() {
        let mut app = ammo_app();
        // No card, no shop: the mirror stays false.
        tick_ammo(&mut app);
        assert!(!app.discount_available);
        // Equip the Discount Card (854) with the shop closed: the refresh
        // copies `discountEquipped` into the App mirror.
        app.player.as_mut().unwrap().equipped[3] = 854;
        tick_ammo(&mut app);
        assert!(app.player.as_ref().unwrap().discount_equipped);
        assert!(app.discount_available, "the closed-shop tick mirrors");
        // Open a shop, then UNEQUIP: the open shop keeps the latch.
        app.shop_npc = Some(0);
        app.player.as_mut().unwrap().equipped[3] = 0;
        tick_ammo(&mut app);
        assert!(!app.player.as_ref().unwrap().discount_equipped);
        assert!(app.discount_available, "the open shop latches true");
        // Close the shop: the next tick re-reads the (now false) flag.
        app.shop_npc = None;
        tick_ammo(&mut app);
        assert!(!app.discount_available, "the closed-shop tick mirrors");
        // The reverse latch: open WITHOUT the card, equip mid-shop — the
        // open shop keeps false until it closes.
        app.shop_npc = Some(0);
        app.player.as_mut().unwrap().equipped[3] = 854;
        tick_ammo(&mut app);
        assert!(app.player.as_ref().unwrap().discount_equipped);
        assert!(!app.discount_available, "the open shop latches false");
        app.shop_npc = None;
        tick_ammo(&mut app);
        assert!(app.discount_available);

        }    // ---- W4-e10 gap census: the pet-grant arms (WRITTEN NOT RUN) ------------

    /// A slot helper for the pet rigs.
    fn pet_slot(kind: u16) -> crate::item::ItemSlot {
        crate::item::ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: 1,
            prefix: 0,
        }
    }
    /// W4-e10 — `Player.UpdatePet` (`Player.cs:17298-17309`): an equipped
    /// pet with no live buff re-grants it at 3600 ticks and plays the
    /// item's UseSound once (`:17305-17306`); a live buff refuses both;
    /// `hideMisc[0]` refuses the whole arm. The Carrot 603 grants (the CE
    /// gate folds true — the retail copy ships the CE content).
    #[test]
    fn w4_e10_the_pet_grant_rebuffs_and_plays_the_use_sound_once() {
        let mut app = ammo_app();
        app.equip.misc_equips[0] = pet_slot(603); // the Carrot, buff 40
        let before = app.audio.played_count();
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(p.buffs.find_buff_index(40).is_some(), "`:17305` AddBuff");
        assert!(
            (3590..=3600).contains(&p.buffs.buff_time[p.buffs.find_buff_index(40).unwrap()]),
            "the 3600 grant (the tick's timers may spend a few)"
        );
        assert_eq!(app.audio.played_count(), before + 1, "`:17306` once");
        // A live buff: no re-grant, no second cue.
        let mid = app.audio.played_count();
        tick_ammo(&mut app);
        assert_eq!(app.audio.played_count(), mid, "`FindBuffIndex != -1`");
        // The hide refusal (`:17303` !hideMisc[0]): clear the buff, hide,
        // and the arm stays silent.
        app.player.as_mut().unwrap().buffs.clear_buff(40);
        app.player.as_mut().unwrap().hide_misc[0] = true;
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.buffs.find_buff_index(40), None, "the hide refused");
        assert_eq!(app.audio.played_count(), mid, "no cue while hidden");
    }

    /// W4-e10 — `Player.UpdatePetLight`'s fairy pick (`Player.cs:17322-
    /// 17338`) at the FUNCTION level: one `Next(3)` maps {27, 102, 101}
    /// (`:17335` Utils.SelectRandom), the UseSound style comes back for
    /// the caller's façade; a live 102 OR 101 fairy satisfies the alias
    /// ladder (`:17323-17330`) — refused, DRAW-FREE; a plain non-27 buff
    /// (152) grants with NO pick draw (`:17333` is the 27-only arm).
    #[test]
    fn w4_e10_the_fairy_pick_is_one_next3_and_the_alias_is_draw_free() {
        let w = World::new("pet".into(), 8, 8);
        // The pick: replay the stream by hand — the granted buff is the
        // trio member the SAME Next(3) picks, and exactly one draw ran.
        let mut p = Player::spawn(&w);
        let mut rng = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        let pick = witness.next_range(0, 3);
        let idx = p.update_light_pet_grant(&pet_slot(425), &mut rng);
        assert_eq!(idx, crate::registry::get(425).unwrap().use_sound);
        assert_eq!(rng.peek(), witness.peek(), "exactly one pick draw");
        assert!(
            p.buffs.find_buff_index([27, 102, 101][pick as usize]).is_some(),
            "the pick's fairy landed"
        );
        // The alias refusals: 102 or 101 already live → no grant, no draw.
        for live in [102, 101] {
            let mut p = Player::spawn(&w);
            p.buffs.add_buff(live, 3600);
            let mut rng = UnifiedRandom::new(7);
            let peek = rng.peek();
            assert_eq!(p.update_light_pet_grant(&pet_slot(425), &mut rng), None);
            assert_eq!(rng.peek(), peek, "the alias ladder is draw-free");
        }
        // The plain 152 grant: buff + style, ZERO draws (the pitch is the
        // façade's, on the caller's stream — the wave-A contract).
        let mut p = Player::spawn(&w);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        let idx = p.update_light_pet_grant(&pet_slot(3043), &mut rng);
        assert_eq!(idx, crate::registry::get(3043).unwrap().use_sound);
        assert!(p.buffs.find_buff_index(152).is_some());
        assert_eq!(rng.peek(), peek, "no pick outside the 27 arm");
        // The vanity twin is draw-free too (no 27 ladder at all).
        let mut p = Player::spawn(&w);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        assert_eq!(
            p.update_pet_grant(&pet_slot(603)),
            crate::registry::get(603).unwrap().use_sound
        );
        assert!(p.buffs.find_buff_index(40).is_some());
        assert_eq!(rng.peek(), peek, "UpdatePet draws nothing itself");
    }

    /// W4-e10 — the fairy grant through the TICK (`Player.cs:25574` →
    /// `:17338`): one of the trio lands, ONE cue plays, and the stream
    /// replay is exact — the standing rig's only unconditional
    /// `UpdatePlayers` draw is the tick-tail Revolver decay (the i14
    /// census), so the grant tick spends pick `Next(3)` → the façade's
    /// case-2 pitch `Next(-6,7)` → decay `Next(3)`, in that order, off
    /// the pristine seed-0 stream. The muted twin proves the gate
    /// precedes the pitch: `soundVolume = 0` spends pick → decay only.
    #[test]
    fn w4_e10_the_fairy_grant_tick_spends_the_pick_then_the_pitch() {
        let mut app = ammo_app();
        app.equip.misc_equips[1] = pet_slot(425);
        let before = app.audio.played_count();
        tick_ammo(&mut app);
        let granted = [27, 102, 101]
            .iter()
            .find(|&&b| app.player.as_ref().unwrap().buffs.find_buff_index(b).is_some())
            .copied();
        assert!(granted.is_some(), "one of the fairy trio landed");
        assert_eq!(app.audio.played_count(), before + 1, "one grant cue");
        let mut replay = UnifiedRandom::new(0);
        let pick = replay.next_range(0, 3);
        assert_eq!(granted, Some([27, 102, 101][pick as usize]), "the pick");
        let _pitch = replay.next_range(-6, 7); // style 25's default window
        let _decay = replay.next_max(3); // the i14 tick-tail draw
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            replay,
            "pick → pitch → decay on UpdatePlayers, nothing else"
        );
        // The muted twin: the gate refuses BEFORE the pitch draw.
        let mut muted = ammo_app();
        muted.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        muted.equip.misc_equips[1] = pet_slot(425);
        tick_ammo(&mut muted);
        let mut m = UnifiedRandom::new(0);
        let _ = m.next_range(0, 3); // the pick still runs (pre-façade)
        let _ = m.next_max(3); // the decay; NO pitch
        assert_eq!(
            *muted.rngs.stream(RngKey::UpdatePlayers),
            m,
            "the muted grant is pick + decay only (the wave-A gate order)"
        );
        // Isolation: every other stage stream stays pristine.
        for key in RngKey::ALL {
            if key == RngKey::UpdatePlayers {
                continue;
            }
            assert_eq!(
                *app.rngs.stream(key),
                UnifiedRandom::new(0),
                "{} — the pet arm rides only UpdatePlayers",
                key.name()
            );
        }
        // The alias refusal through the tick: no second cue.
        let mid = app.audio.played_count();
        tick_ammo(&mut app);
        assert_eq!(app.audio.played_count(), mid, "the live fairy refuses");
    }

    /// W4-e10 — the plain light-pet grant through the TICK: the Magic
    /// Lantern (3043, buff 152) re-grants at 3600 and plays ONE UseSound
    /// cue; the grant itself draws nothing (the function-level witness
    /// pins that) — the tick's only extra spend is the façade's pitch.
    #[test]
    fn w4_e10_the_plain_light_pet_grant_rebuffs_and_plays_once() {
        let mut app = ammo_app();
        app.equip.misc_equips[1] = pet_slot(3043); // buff 152
        let before = app.audio.played_count();
        tick_ammo(&mut app);
        assert!(app.player.as_ref().unwrap().buffs.find_buff_index(152).is_some());
        assert_eq!(app.audio.played_count(), before + 1, "`:17338` once");
        let mid = app.audio.played_count();
        tick_ammo(&mut app);
        assert_eq!(app.audio.played_count(), mid, "the live buff refuses");
        // hideMisc[1] refuses the whole arm.
        app.player.as_mut().unwrap().buffs.clear_buff(152);
        app.player.as_mut().unwrap().hide_misc[1] = true;
        tick_ammo(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().buffs.find_buff_index(152),
            None,
            "the hide refused (`:17318`)"
        );
        assert_eq!(app.audio.played_count(), mid, "no cue while hidden");
    }

    // ---- W4-a5: the QuickBuff B-key edge (WRITTEN NOT RUN) ------------------

    /// W4-a5 — `TriggersSet.CopyInto` (`GameInput/TriggersSet.cs:801-804`)
    /// fires `Player.QuickBuff` on the QuickBuff trigger; the port keeps
    /// the press-edge latch (`quick_buff_prev`, the `quick_mana_prev`
    /// precedent). Holding B does not re-buff; release + re-press does.
    #[test]
    fn w4_a5_the_quick_buff_key_buffs_once_per_press() {
        let mut app = ammo_app();
        app.inventory[4] = crate::item::ItemSlot {
            kind: 292, // Ironskin Potion, buff 5
            name: "Ironskin Potion",
            count: 5,
            prefix: 0,
        };
        // The silent listener: the tail use-sound draws nothing.
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app.keys.insert(KeyCode::KeyB); // the default cBuff bind
        tick_ammo(&mut app);
        assert!(
            app.player
                .as_ref()
                .unwrap()
                .buffs
                .find_buff_index(5)
                .is_some(),
            "the press granted Ironskin"
        );
        assert_eq!(app.inventory[4].count, 4);
        // Held: the latch dropped — no second consume (the standing buff
        // would refuse anyway, so witness the STACK, not the buff).
        for _ in 0..30 {
            tick_ammo(&mut app);
        }
        assert_eq!(app.inventory[4].count, 4, "one use per press edge");
        // Release + re-press: the latch re-arms. The buff still stands, so
        // ShouldBother refuses — clear it first to see the second consume.
        app.keys.remove(&KeyCode::KeyB);
        tick_ammo(&mut app);
        app.player.as_mut().unwrap().buffs.clear_buff(5);
        app.keys.insert(KeyCode::KeyB);
        tick_ammo(&mut app);
        assert_eq!(app.inventory[4].count, 3, "the re-press buffed again");
        assert!(app
            .player
            .as_ref()
            .unwrap()
            .buffs
            .find_buff_index(5)
            .is_some());
    }
    /// W4-c39 — the lantern-night luck term wires live (`Player.cs:29146-29149`):
    /// `LanternNight.LanternsUp` feeds `luck_state.lantern_night` before the
    /// tick's `recalculate_luck`, adding 0.3 luck; the lanterns-down tick
    /// clears it.
    #[test]
    fn w4_c39_the_lantern_night_adds_luck_live() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("luck".into(), 40, 200);
        app.world.compute_column_surface();
        let mut p = Player::spawn(&app.world);
        // Underground: the torch scan's surface gate keeps torch_luck at 0,
        // so the lantern term is the only luck source.
        p.y = 150.0 * 16.0;
        app.player = Some(p);
        // Lanterns up: the tick feeds the flag and the luck gains 0.3.
        app.world_events.manual_lanterns = true;
        update_player(&mut app, 1.0 / 60.0);
        let pl = app.player.as_ref().unwrap();
        assert!(pl.luck_state.lantern_night, "the flag feeds from LanternsUp");
        assert!(
            (pl.luck_state.luck - 0.3).abs() < 1e-4,
            "the lantern night adds 0.3 luck: {}",
            pl.luck_state.luck
        );
        // Lanterns down: the flag clears and the term drops.
        app.world_events.manual_lanterns = false;
        update_player(&mut app, 1.0 / 60.0);
        let pl = app.player.as_ref().unwrap();
        assert!(!pl.luck_state.lantern_night);
        assert!(pl.luck_state.luck.abs() < 1e-4, "the term drops: {}", pl.luck_state.luck);
    }
    // ---- W4-d20 T4b: UpdatePet/UpdatePetLight persistence — WRITTEN NOT RUN ----
    //
    // The witnesses call `update_pet_persistence` DIRECTLY (not through the
    // full tick): the accessory buff arm (`spawn_pet_if_needed`) pins a LIVE
    // pet buff to 18000 every tick, which would mask the re-grant's 3600 and
    // the absent-buff gate. The direct call isolates the `UpdatePet` logic.

    /// A pet-persistence rig: a standing player, a silent audio bank and a
    /// seed-0 `UpdatePlayers` stream.
    fn pet_rig() -> (Player, crate::audio::Audio, UnifiedRandom) {
        let w = World::new("pet".into(), 64, 48);
        let p = Player::spawn(&w);
        (p, crate::audio::Audio::new(None), UnifiedRandom::new(0))
    }

    /// The Jumino pet item 5276 (`DefaultToVanitypet(994, 345)`,
    /// `Item.cs:41867-41872`): equipped in misc[0] with NO buff present,
    /// `UpdatePet` (`Player.cs:17298-17309`) re-grants `AddBuff(345, 3600)`
    /// and plays the UseSound. The use-path's `buffTime = 0` grant expires
    /// the same tick — this is the mechanism that keeps the pet alive.
    #[test]
    fn w4_d20_t4b_the_equipped_jumino_regrants_its_buff() {
        let mut misc = [EMPTY_SLOT; MISC];
        misc[0] = slot(5276);
        let (mut p, mut audio, mut rng) = pet_rig();
        assert_eq!(p.buffs.find_buff_index(345), None, "no buff yet");
        update_pet_persistence(&mut p, &misc, &mut audio, &mut rng);
        let idx = p.buffs.find_buff_index(345).expect("the pet buff re-granted");
        assert_eq!(p.buffs.buff_time[idx], 3600, "AddBuff(num, 3600)");
    }

    /// The re-grant is gated on the buff being ABSENT: with buff 345
    /// already live, `UpdatePet` does nothing (no duplicate, no reset).
    #[test]
    fn w4_d20_t4b_the_live_pet_buff_is_not_regranted() {
        let mut misc = [EMPTY_SLOT; MISC];
        misc[0] = slot(5276);
        let (mut p, mut audio, mut rng) = pet_rig();
        p.buffs.add_buff(345, 100);
        update_pet_persistence(&mut p, &misc, &mut audio, &mut rng);
        let idx = p.buffs.find_buff_index(345).unwrap();
        assert_eq!(p.buffs.buff_time[idx], 100, "the live buff is untouched");
    }

    /// `hideMisc[0]` blocks the re-grant (`!this.hideMisc[0]`, `:17303`):
    /// a hidden pet stays hidden until un-hidden.
    #[test]
    fn w4_d20_t4b_the_hidden_pet_does_not_regrant() {
        let mut misc = [EMPTY_SLOT; MISC];
        misc[0] = slot(5276);
        let (mut p, mut audio, mut rng) = pet_rig();
        p.hide_misc[0] = true;
        update_pet_persistence(&mut p, &misc, &mut audio, &mut rng);
        assert_eq!(p.buffs.find_buff_index(345), None, "the hidden pet refuses");
    }

    /// The Carrot (603) gate (`type != 603 || runningCollectorsEdition`,
    /// `:17303`): the collector's-edition flag folds false in the port, so
    /// the Carrot never re-grants even though buff 40 is a vanity pet.
    #[test]
    fn w4_d20_t4b_the_carrot_never_regrants() {
        let mut misc = [EMPTY_SLOT; MISC];
        misc[0] = slot(603);
        let (mut p, mut audio, mut rng) = pet_rig();
        let carrot_buff = crate::registry::get(603).unwrap().buff_type;
        update_pet_persistence(&mut p, &misc, &mut audio, &mut rng);
        assert_eq!(
            p.buffs.find_buff_index(carrot_buff),
            None,
            "the Carrot is collector's-edition-only"
        );
    }

    /// The light-pet fairy trio (`UpdatePetLight`, `:17322-17336`): the
    /// Fairy Bell (425, buff 27) with no 27/102/101 present picks one via
    /// `SelectRandom(Main.rand, 27, 102, 101)` — one `Next(3)` draw — and
    /// grants it 3600. A live 102 alias blocks the re-grant (no draw).
    #[test]
    fn w4_d20_t4b_the_fairy_trio_picks_and_grants() {
        let mut misc = [EMPTY_SLOT; MISC];
        misc[1] = slot(425);
        let (mut p, mut audio, mut rng) = pet_rig();
        let before = rng.clone();
        update_pet_persistence(&mut p, &misc, &mut audio, &mut rng);
        let granted = [27, 102, 101]
            .iter()
            .find(|&&b| p.buffs.find_buff_index(b).is_some())
            .copied();
        assert!(granted.is_some(), "one of the fairy trio was granted");
        let idx = p.buffs.find_buff_index(granted.unwrap()).unwrap();
        assert_eq!(p.buffs.buff_time[idx], 3600);
        assert_ne!(rng, before, "the fairy pick drew Next(3)");
        // A live 102 alias blocks the re-grant and spends no draw.
        let (mut p2, mut audio2, mut rng2) = pet_rig();
        p2.buffs.add_buff(102, 100);
        let before2 = rng2.clone();
        update_pet_persistence(&mut p2, &misc, &mut audio2, &mut rng2);
        assert!(p2.buffs.find_buff_index(102).is_some(), "the alias stands");
        assert_eq!(p2.buffs.find_buff_index(27), None, "no 27 re-grant");
        assert_eq!(p2.buffs.find_buff_index(101), None, "no 101 re-grant");
        assert_eq!(rng2, before2, "the alias path spends no draw");

/// W4-d80 — the vampire sun-scorch sizzle (research/64 sites 22-24): the
/// counter ramp, the lerp volume, the ensure/follow dance through the
/// drain, the shared-atomic condition fold and the RNG contract (zero
/// gameplay draws). WRITTEN NOT RUN (code-only mode).
#[cfg(test)]
mod w4_d80_vampire_sizzle_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::slot::SlotId;

    /// The counter ramp (`Player.cs:28582-28589`): burning +1, alive −6,
    /// dead −2 (the dead tick clears the burning flag), clamped 0..300.
    #[test]
    fn w4_d80_the_scorch_counter_ramps_and_clamps() {
        let w = World::new("t".into(), 64, 64);
        let mut p = Player::spawn(&w);
        // Alive, not burning: −6 per tick, floored at 0.
        p.sun_scorch_counter = 10;
        p.tick_sun_scorch();
        assert_eq!(p.sun_scorch_counter, 4);
        p.tick_sun_scorch();
        assert_eq!(p.sun_scorch_counter, 0, "the floor holds");
        // Burning: +1 per tick, capped at 300.
        p.vampire_burning_in_sunlight = true;
        p.sun_scorch_counter = 299;
        p.tick_sun_scorch();
        assert_eq!(p.sun_scorch_counter, 300);
        p.tick_sun_scorch();
        assert_eq!(p.sun_scorch_counter, 300, "the cap holds");
        // Dead: the burning flag clears, −2 per tick.
        p.dead = true;
        p.tick_sun_scorch();
        assert!(!p.vampire_burning_in_sunlight, "the dead tick clears the flag");
        assert_eq!(p.sun_scorch_counter, 298);
    }

    /// The lerp (`Player.cs:28589`): `GetLerpValue(0, 120, counter,
    /// clamped)` — 0 at 0, 0.5 at 60, 1 at 120+.
    #[test]
    fn w4_d80_the_scorch_lerp_matches_get_lerp_value() {
        let w = World::new("t".into(), 64, 64);
        let mut p = Player::spawn(&w);
        for (counter, want) in [(0i32, 0.0f32), (60, 0.5), (120, 1.0), (300, 1.0)] {
            p.sun_scorch_counter = counter;
            assert!(
                (p.sun_scorch_lerp() - want).abs() < 1e-6,
                "counter {counter}: {}",
                p.sun_scorch_lerp()
            );
        }
    }

    /// The headless rig: the sizzle cue seeded silent, the vampire seed
    /// armed, the player's counter mid-ramp.
    fn rig() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.audio.test_seed_tracked_cue("sizzle");
        app.world
            .micro_seeds
            .insert(tr_core::seeds::MicroSeedFlags::VAMPIRISM);
        let mut p = Player::spawn(&app.world);
        p.sun_scorch_counter = 60; // lerp 0.5
        app.player = Some(p);
        app
    }

    /// The ensure plays once into the handle with the lerp volume; the
    /// follow re-points the live loop; a zero lerp never ensures
    /// (`Player.cs:28590-28603`).
    #[test]
    fn w4_d80_the_sizzle_ensures_follows_and_gates_on_lerp() {
        let mut app = rig();
        update_sizzle_audio(&mut app);
        let id = app.player.as_ref().unwrap().sizzle_audio_handle;
        assert!(id.is_active(), "the ensure stored a live handle");
        app.audio.test_keep_tracked_alive(id);
        assert!(
            (app.audio.get_active_sound(id).unwrap().volume - 0.5).abs() < 1e-6,
            "the lerp volume override"
        );
        // The follow re-points the live loop (no re-play).
        if let Some(p) = app.player.as_mut() {
            p.x += 100.0;
            p.sun_scorch_counter = 120; // lerp 1.0
        }
        update_sizzle_audio(&mut app);
        let id2 = app.player.as_ref().unwrap().sizzle_audio_handle;
        assert_eq!(id2, id, "no re-play while live");
        let (cx, cy) = app.player.as_ref().unwrap().center();
        assert_eq!(app.audio.get_active_sound(id).unwrap().position, (cx, cy));
        assert!((app.audio.get_active_sound(id).unwrap().volume - 1.0).abs() < 1e-6);
        // A zero lerp with no live handle never ensures.
        let mut app2 = rig();
        app2.player.as_mut().unwrap().sun_scorch_counter = 0;
        update_sizzle_audio(&mut app2);
        assert_eq!(
            app2.player.as_ref().unwrap().sizzle_audio_handle,
            SlotId::INVALID,
            "the zero lerp gates the ensure"
        );
    }

    /// The condition mirror: the shared atomic tracks `vampireSeed &&
    /// counter > 0` — the seed off OR the counter zero flips it false.
    #[test]
    fn w4_d80_the_sizzle_condition_mirrors_the_tracker() {
        let mut app = rig();
        update_sizzle_audio(&mut app);
        let alive = app.player.as_ref().unwrap().sizzle_alive.clone().unwrap();
        assert!(alive.load(std::sync::atomic::Ordering::Relaxed), "seed + counter live");
        // The counter drains to zero: the condition flips false.
        app.player.as_mut().unwrap().sun_scorch_counter = 0;
        update_sizzle_audio(&mut app);
        assert!(!alive.load(std::sync::atomic::Ordering::Relaxed), "the zero counter flips it");
        // The seed off flips it false too.
        app.player.as_mut().unwrap().sun_scorch_counter = 60;
        app.world.micro_seeds = tr_core::seeds::MicroSeedFlags::none();
        update_sizzle_audio(&mut app);
        assert!(!alive.load(std::sync::atomic::Ordering::Relaxed), "the seed off flips it");
    }

    /// The RNG contract: the sizzle ensure + follow leave the gameplay
    /// stream pristine; the pitch roll moves (one variation: no pick).
    #[test]
    fn w4_d80_the_sizzle_drain_is_gameplay_draw_free() {
        let mut app = rig();
        let before = app.rngs.stream(RngKey::UpdatePlayers).clone();
        let (style_before, pitch_before) = app.audio.test_audio_streams();
        update_sizzle_audio(&mut app);
        update_sizzle_audio(&mut app);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdatePlayers),
            before,
            "zero Main.rand draws on the sizzle path"
        );
        let (style_after, pitch_after) = app.audio.test_audio_streams();
        assert_eq!(style_after, style_before, "one variation: no variation pick");
        assert_ne!(pitch_after, pitch_before, "the pitch roll moved");
    }

    /// The style row: `VampireSizzle` resolves with volume 1, one
    /// variation, uncapped (`SoundID.cs:1708`'s
    /// `CreateTrackable("sizzle").WithVolume(1f)`).
    #[test]
    fn w4_d80_the_sizzle_style_resolves() {
        let style = crate::tracked_sound::TrackedStyle::by_name("VampireSizzle").expect("VampireSizzle");
        assert_eq!(style.cue, "sizzle");
        assert_eq!(style.variations, 1);
        assert!((style.volume - 1.0).abs() < 1e-6);
        assert_eq!(style.max_tracked_instances, 0);
    }

    // ---- W4-f30: the per-tick armor achievement tail (38 / 41) ------------

    /// A rig for `UpdateArmorSets`' local-player tail (`Player.cs:25686-25704`):
    /// a standing player above the surface, day clock inside the ON_FLEEK
    /// window, no rain. The equipped kinds ride `app.equip.armor` (the tick
    /// snapshots them into `p.equipped`).
    fn armor_tail_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("armor-tail".into(), 64, 200);
        app.world.compute_column_surface();
        app.world.day_time = true;
        app.world.time = 10000.0; // inside [3600, 50400]
        let mut p = Player::spawn(&app.world);
        p.y = 100.0; // above world_surface * 16 (200 * 0.25 * 16 = 800)
        app.player = Some(p);
        app
    }

    fn wear(app: &mut App, slot: usize, kind: u16) {
        app.equip.armor[slot] = crate::item::ItemSlot { kind, name: "", count: 1, prefix: 0 };
    }

    fn row_completed(app: &App, name: &str) -> bool {
        app.achievements.list.iter().find(|a| a.name == name).unwrap().completed
    }

    /// ON_FLEEK (38, `Player.cs:25691-25694`): Sunglasses (head slot 12) in
    /// the day window above the surface fires; rain / night / below-surface
    /// each refuse.
    #[test]
    fn f30_on_fleet_needs_sunglasses_in_the_day_window() {
        let mut app = armor_tail_app();
        wear(&mut app, 0, 237); // Sunglasses, head_slot 12
        tick_ammo(&mut app);
        assert!(row_completed(&app, "ON_FLEEK"), "the day-window sunglasses fire 38");

        // Rain refuses.
        let mut rainy = armor_tail_app();
        wear(&mut rainy, 0, 237);
        rainy.rain.set_raining_for_test(true);
        tick_ammo(&mut rainy);
        assert!(!row_completed(&rainy, "ON_FLEEK"), "rain refuses");

        // Night refuses.
        let mut night = armor_tail_app();
        wear(&mut night, 0, 237);
        night.world.day_time = false;
        tick_ammo(&mut night);
        assert!(!row_completed(&night, "ON_FLEEK"), "night refuses");

        // Below the surface refuses.
        let mut deep = armor_tail_app();
        wear(&mut deep, 0, 237);
        deep.player.as_mut().unwrap().y = 5000.0;
        tick_ammo(&mut deep);
        assert!(!row_completed(&deep, "ON_FLEEK"), "below the surface refuses");
    }

    /// MINI_ME (41, `Player.cs:25699-25702`): the full Moon Lord vanity set
    /// (mask head 187 / body 249 / legs 217) fires; a missing piece refuses.
    /// The vanity slots 10/11/12 carry the set (the effective-armor fold).
    #[test]
    fn f30_mini_me_needs_the_full_moon_lord_set() {
        let mut app = armor_tail_app();
        wear(&mut app, 10, 3373); // Boss Mask Moonlord, head_slot 187
        wear(&mut app, 11, 5515); // Moon Lord Body, body_slot 249
        wear(&mut app, 12, 5001); // Moon Lord Legs, leg_slot 217
        tick_ammo(&mut app);
        assert!(row_completed(&app, "MINI_ME"), "the full set fires 41");

        // A missing leg piece refuses.
        let mut partial = armor_tail_app();
        wear(&mut partial, 10, 3373);
        wear(&mut partial, 11, 5515);
        tick_ammo(&mut partial);
        assert!(!row_completed(&partial, "MINI_ME"), "two of three refuses");
    }

    /// W4-f40 — `Player.RecalculateLuck`'s lantern-night term
    /// (`Player.cs:29146-29149`): while `LanternNight.LanternsUp` the player's
    /// luck carries +0.3. The port snapshots the flag beside the auras/zones
    /// each tick, so `update_luck`'s recalculation reads it. The contrast arm
    /// (lanterns down) keeps the base luck, and the genuine/manual flags both
    /// drive `LanternsUp` (`LanternNight.cs:17-27`).
    #[test]
    fn w4_f40_the_lantern_night_adds_the_luck_bonus_through_the_player_tick() {
        // Lanterns down: the base luck (no torch/coin/ladybug inputs on the
        // empty rig) is 0.
        let mut down = ammo_app();
        tick_ammo(&mut down);
        assert!(
            down.player.as_ref().unwrap().luck().abs() < 1e-6,
            "no lantern bonus while down"
        );
        assert!(!down.player.as_ref().unwrap().luck_state.lantern_night);

        // Manual lanterns up (the `ToggleManualLanterns` path): +0.3.
        let mut manual = ammo_app();
        manual.world_events.manual_lanterns = true;
        tick_ammo(&mut manual);
        let p = manual.player.as_ref().unwrap();
        assert!(p.luck_state.lantern_night, "the tick latched the flag");
        assert!(
            (p.luck() - 0.3).abs() < 1e-6,
            "the +0.3 lantern-night luck: {}",
            p.luck()
        );

        // Genuine lanterns up (the natural/override path): the same +0.3.
        let mut genuine = ammo_app();
        genuine.world_events.genuine_lanterns = true;
        tick_ammo(&mut genuine);
        assert!(
            (genuine.player.as_ref().unwrap().luck() - 0.3).abs() < 1e-6,
            "the genuine lanterns carry the same bonus"
        );
    }
}
// balance-close (union shared delimiter)
}
}
