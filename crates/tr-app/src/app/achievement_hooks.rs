//! The achievement equip/enter-world hooks — `AchievementsHelper.HandleOnEquip`
//! (`GameContent/Achievements/AchievementsHelper.cs:239-280`) and
//! `AchievementsHelper.OnPlayerEnteredWorld` (`:94-198`, armed by `Initialize`
//! `:89-92` on `Player.Hooks.OnEnterWorld`). W4-c36.
//!
//! Every completion names ONE row's condition through
//! `AchievementManager::complete_condition` (the C#
//! `Main.Achievements.GetCondition(row, cond).Complete()`), never a flag-wide
//! broadcast. The `ItemSlot` call sites (`ItemSlot.cs:972/1039/1102/1143/
//! 1205/1895` and `SwapEquip` `:3297/:3307/:3317/:3355`) pass the slot's
//! context id; the port's callers (`app/clicks.rs`'s equip ladder) pass the
//! same ids.
//!
//! Deviations (recorded in docs/ANALYSIS.md "Achievement notify sites
//! (W4-c36)"): the `OnItemPickup` inventory/armor/dye/bank sweeps of
//! `OnPlayerEnteredWorld` (`:96-147`) fire only for the arrays the port
//! carries — the `Loadouts` walk (`:134-146`) has no loadout system;
//! `WorldGen.CheckAchievement_RealEstateAndTownSlimes` (`:196`) and
//! `TryGrantingBestiary100PercentAchievement` (`:197`) ride their own seams.

use crate::app::App;
use crate::item::ItemSlot;
use crate::player::Player;

/// `AchievementsHelper.HandleOnEquip(player, item, context)`
/// (`AchievementsHelper.cs:239-280`). `item` is the slot's NEW content (the
/// C# reads `inv[slot]` after the swap); the armor/dye reads are the live
/// arrays AFTER the write, exactly as the C# call sites see them.
pub(crate) fn handle_on_equip(app: &mut App, item: ItemSlot, context: i32) {
    // `:241-244`: the grapple slot (miscEquips[4], context 16).
    if context == 16 {
        let _ = app.achievements.complete_condition("HOLD_ON_TIGHT", "Equip");
    }
    // `:245-248`: the mount slot (miscEquips[3], context 17).
    if context == 17 {
        let _ = app.achievements.complete_condition("THE_CAVALRY", "Equip");
    }
    // `:249-252`: an accessory/vanity-accessory slot holding wings. The
    // `item.wingSlot > 0` read is the port's `Player::wing_slot` census.
    if (context == 10 || context == 11) && Player::wing_slot(item.kind as i32) > 0 {
        let _ = app.achievements.complete_condition("HEAD_IN_THE_CLOUDS", "Equip");
    }
    // `:253-256`: the third real armour piece completes the set.
    if context == 8
        && app.equip.armor[0].count > 0
        && app.equip.armor[1].count > 0
        && app.equip.armor[2].count > 0
    {
        let _ = app.achievements.complete_condition("MATCHING_ATTIRE", "Equip");
    }
    // `:257-260`: the vanity trio (armor[10..12]).
    if context == 9
        && app.equip.armor[10].count > 0
        && app.equip.armor[11].count > 0
        && app.equip.armor[12].count > 0
    {
        let _ = app.achievements.complete_condition("FASHION_STATEMENT", "Equip");
    }
    // `:261-264`: only the dye contexts run the DYE_HARD ladder.
    if context != 12 && context != 33 {
        return;
    }
    let Some(p) = app.player.as_ref() else { return };
    // `:265-271`: every unlocked dye slot must hold a dye.
    for i in 0..10 {
        if dye_slot_unlocked(p, i)
            && (app.equip.dye[i].kind < 1 || app.equip.dye[i].count < 1)
        {
            return;
        }
    }
    // `:272-278`: the five misc dyes too (no unlock gate on this walk).
    for slot in &app.equip.misc_dyes {
        if slot.kind < 1 || slot.count < 1 {
            return;
        }
    }
    let _ = app.achievements.complete_condition("DYE_HARD", "Equip");
}

/// `Player.IsItemSlotUnlockedAndUsable` (`Player.cs:12579-12604`) — the same
/// gate `app/equip_swap.rs` applies: 8/18 need the Demon Heart + expert,
/// 9/19 need master (unported → locked), the `Main.gameMenu` escapes read
/// false in play.
fn dye_slot_unlocked(p: &Player, slot: usize) -> bool {
    match slot {
        8 | 18 => p.extra_accessory && p.buffs.expert_mode,
        9 | 19 => false,
        _ => true,
    }
}

/// `AchievementsHelper.OnPlayerEnteredWorld` (`AchievementsHelper.cs:94-198`):
/// the enter-world sweep. The `OnItemPickup` fan-out (`:96-147`) rides the
/// port's `ItemPickup` event shape over every carried array; the flag block
/// (`:148-195`) completes the named rows off the loaded stats/equips.
pub(crate) fn on_player_entered_world(app: &mut App) {
    use crate::achievements::CondKind;
    // `:98-133`: inventory 0..58 (grid + coins + ammo + trash), armor, dye,
    // miscEquips, miscDyes, bank..bank4. The `Loadouts` walk (`:134-146`)
    // has no port system — recorded deviation.
    let mut kinds: Vec<u16> = Vec::new();
    kinds.extend(app.inventory.iter().map(|s| s.kind));
    kinds.extend(app.equip.coins.iter().map(|s| s.kind));
    kinds.extend(app.equip.ammo.iter().map(|s| s.kind));
    kinds.push(app.equip.trash.kind);
    kinds.extend(app.equip.armor.iter().map(|s| s.kind));
    kinds.extend(app.equip.dye.iter().map(|s| s.kind));
    kinds.extend(app.equip.misc_equips.iter().map(|s| s.kind));
    kinds.extend(app.equip.misc_dyes.iter().map(|s| s.kind));
    if let Some(p) = app.player.as_ref() {
        for bank in &p.banks {
            kinds.extend(bank.iter().map(|s| s.kind));
        }
    }
    for kind in kinds {
        // The C# fires the event for air slots too (`(short)item.type` = 0);
        // no row listens on id 0, so the dispatch is the same no-op.
        let _ = app.achievements.notify(CondKind::ItemPickup(kind));
    }
    let Some(p) = app.player.as_ref() else { return };
    // `:148-151`: STAR_POWER on any mana investment.
    if p.mana_max_base > 20 {
        let _ = app.achievements.complete_condition("STAR_POWER", "Use");
    }
    // `:152-155`: TOPPED_OFF at the 500/200 caps.
    if p.max_hp == 500 && p.mana_max_base == 200 {
        let _ = app.achievements.complete_condition("TOPPED_OFF", "Use");
    }
    // `:156-163`: the grapple/mount slots.
    if app.equip.misc_equips[4].kind > 0 {
        let _ = app.achievements.complete_condition("HOLD_ON_TIGHT", "Equip");
    }
    if app.equip.misc_equips[3].kind > 0 {
        let _ = app.achievements.complete_condition("THE_CAVALRY", "Equip");
    }
    // `:164-171`: any worn wings (the whole armor array — the vanity
    // accessories count, the C# walks `player.armor.Length`).
    if app
        .equip
        .armor
        .iter()
        .any(|s| Player::wing_slot(s.kind as i32) > 0)
    {
        let _ = app.achievements.complete_condition("HEAD_IN_THE_CLOUDS", "Equip");
    }
    // `:172-179`: the armour/vanity trios.
    if app.equip.armor[0].count > 0
        && app.equip.armor[1].count > 0
        && app.equip.armor[2].count > 0
    {
        let _ = app.achievements.complete_condition("MATCHING_ATTIRE", "Equip");
    }
    if app.equip.armor[10].count > 0
        && app.equip.armor[11].count > 0
        && app.equip.armor[12].count > 0
    {
        let _ = app.achievements.complete_condition("FASHION_STATEMENT", "Equip");
    }
    // `:180-191`: DYE_HARD — the enter-world form checks ONLY the ten dye
    // slots (the misc-dye walk is `HandleOnEquip`'s, `:272-278`).
    let mut dyed = true;
    for i in 0..10 {
        if dye_slot_unlocked(p, i)
            && (app.equip.dye[i].kind < 1 || app.equip.dye[i].count < 1)
        {
            dyed = false;
        }
    }
    if dyed {
        let _ = app.achievements.complete_condition("DYE_HARD", "Equip");
    }
    // `:192-195`: the Torch God's favour persists on the player file.
    if p.unlocked_biome_torches {
        let _ = app.achievements.complete_condition("GAIN_TORCH_GODS_FAVOR", "Use");
    }
    // `:196-197`: the real-estate/town-slime roster check and the bestiary
    // 100% grant ride their own seams (`world_sim/npcs.rs`'s arrival pass,
    // `App::try_grant_bestiary_100_net`) — not re-fired here.
}

/// W4-c36 DOC-SWEEP — `WorldGen.CheckAchievement_RealEstateAndTownSlimes`
/// (`WorldGen.cs:5659-5677`): the roster scan marks every active, non-homeless
/// NPC's type (no `townNPC` filter — the C# array is `NPCID.Count` wide), then
/// the 25-id housed roster fires progression 17 (REAL_ESTATE_AGENT) and the
/// 8 town slimes fire 26 (ALL_TOWN_SLIMES). Both tests are independent `if`s.
/// The port's roster is partial: Truffle 160, Steampunker 178, Cyborg 209 and
/// Pirate 229 (four kinds) plus all eight town slimes (670/678-684) have no
/// NpcKind rows, so both rows are unreachable in play until those kinds land
/// — the wire is the faithful check, cited like the blood-moon no-op wires.
/// DD2 Bartender 550 IS ported (`NpcKind::DD2Bartender`, bestiary `net_id:
/// 550`), so it is NOT a missing roster id (the W4-c36 re-probe corrected the
/// earlier "five missing kinds" census). The scan reads `n.kind.net_id()`,
/// which resolves through `bestiary::ENTRIES` (`npc/data.rs:1186`) — there is
/// NO hardcoded id list, so the check activates automatically the moment the
/// d36/d39/d41/d43 kind bands land their bestiary rows. `npc.home.is_none()`
/// is the port's `homeless` flag (`world_sim/npcs.rs`'s arrival writes it).
pub(crate) fn check_real_estate_and_town_slimes(app: &mut App) {
    let mut housed: Vec<i32> = app
        .npcs
        .iter()
        .filter(|n| n.hp > 0 && n.home.is_some())
        .map(|n| n.kind.net_id())
        .collect();
    housed.sort_unstable();
    housed.dedup();
    // `:5668` — the 25-id roster, in the C#'s array-test order.
    const ROSTER: [i32; 25] = [
        38, 17, 107, 19, 22, 124, 228, 178, 18, 229, 209, 54, 108, 160, 20, 369, 207, 227, 208,
        441, 353, 550, 588, 633, 663,
    ];
    if ROSTER.iter().all(|id| housed.contains(id)) {
        // `:5671` — REAL_ESTATE_AGENT.
        let _ = app.notify_progression_net(17);
    }
    // `:5673` — the eight town slimes.
    const SLIMES: [i32; 8] = [670, 678, 679, 680, 681, 682, 683, 684];
    if SLIMES.iter().all(|id| housed.contains(id)) {
        // `:5675` — ALL_TOWN_SLIMES.
        let _ = app.notify_progression_net(26);
    }
}

/// W4-c36 — the per-tick reach/biome checks: `Player.UpdateBiomes`' tail
/// (`Player.cs:16732-16776`) + `BordersMovement`'s two achievement reads
/// (`:24066-24126`). Returns the `(row, condition)` pairs to complete — the
/// caller fires them after the player borrow releases the manager. The
/// funkytown cooldown (`_funkytownAchievementCheckCooldown`, `:16752-16775`)
/// rides the player: alive ticks decrement it, a dead tick re-arms 100.
pub(crate) fn reach_check(
    p: &mut Player,
    world: &tr_core::World,
    zone_graveyard: bool,
) -> Vec<(&'static str, &'static str)> {
    let mut out = Vec::new();
    // The C# gates on `!this.dead`; the port latches `dead` in the later
    // npcs-pass death seam, so `hp <= 0` stands in for the synchronous
    // `KillMe` death on the lethal tick itself.
    if p.dead || p.hp <= 0 {
        // `:16773-16776`: the dead arm re-arms the cooldown.
        p.funkytown_achievement_check_cooldown = 100;
        return out;
    }
    // `:16734-16751`: the centre tile's wall — 86 (hive) fires event 12,
    // 62 (spider) event 13.
    let (cx, cy) = p.center();
    let (tx, ty) = ((cx / 16.0) as i32, (cy / 16.0) as i32);
    if world.in_bounds(tx, ty) {
        let wall = world.tile(tx, ty).map_or(0, |t| t.wall);
        match wall {
            86 => out.push(("WHERES_MY_HONEY", "Reach")),
            62 => out.push(("JEEPERS_CREEPERS", "Reach")),
            _ => {}
        }
    }
    // `:16752-16755`: the cooldown tick.
    if p.funkytown_achievement_check_cooldown > 0 {
        p.funkytown_achievement_check_cooldown -= 1;
    }
    // `:16756-16759`: `Main.specialSeedWorld` (`Main.cs:2650-2660`) — any of
    // the nine secret-seed bits.
    if world.seeds.bits() != 0 {
        out.push(("PLAY_ON_A_SPECIAL_SEED", "Do"));
    }
    // `:16760-16771`: the underworld / surface-glowshroom / graveyard ladder
    // (the `else if` chain — one arm per tick). `Main.UnderworldLayer` is the
    // port's `height - 200` convention (`worldgen/biomes.rs:468`).
    let y_tiles = p.y / 16.0;
    let underworld_layer = (world.height - 200) as f32;
    if y_tiles > underworld_layer {
        out.push(("ITS_GETTING_HOT_IN_HERE", "Reach"));
    } else if p.funkytown_achievement_check_cooldown == 0
        && (y_tiles as f64) < world.world_surface
        && p.zones.glowshroom
    {
        out.push(("FUNKYTOWN", "Reach"));
    } else if p.funkytown_achievement_check_cooldown == 0 && zone_graveyard {
        out.push(("FOUND_GRAVEYARD", "Reach"));
    }
    // `BordersMovement` (`:24066-24099`): the top border (topWorld + 640)
    // fires event 11; `:24123-24126`: the bottom band (bottomWorld - 640 -
    // 118 - height) fires event 10. The port carries no 640-px border clamp
    // (its own bounds seam, `player.rs`), but the achievement reads are the
    // same position tests.
    if p.y < 640.0 {
        out.push(("INTO_ORBIT", "Reach"));
    }
    if p.y > world.height as f32 * 16.0 - 640.0 - 118.0 - p.height {
        out.push(("ROCK_BOTTOM", "Reach"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    fn app_with_player() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        app
    }

    fn slot(kind: u16) -> ItemSlot {
        ItemSlot { kind, name: "", count: 1, prefix: 0 }
    }

    fn done(app: &App, name: &str) -> bool {
        app.achievements.list.iter().any(|a| a.name == name && a.completed)
    }

    /// `AchievementsHelper.cs:241-248` — the grapple/mount contexts complete
    /// their rows; a dye context does not.
    #[test]
    fn the_grapple_and_mount_contexts_complete_their_rows() {
        let mut app = app_with_player();
        handle_on_equip(&mut app, slot(84), 16);
        assert!(done(&app, "HOLD_ON_TIGHT"));
        assert!(!done(&app, "THE_CAVALRY"));
        handle_on_equip(&mut app, slot(1914), 17);
        assert!(done(&app, "THE_CAVALRY"));
    }

    /// `:249-260` — wings on ctx 10/11, the armour trio on ctx 8, the vanity
    /// trio on ctx 9; each gate reads the LIVE arrays after the write.
    #[test]
    fn the_wings_and_attire_gates_read_the_live_arrays() {
        let mut app = app_with_player();
        // The wing item id from the port's census (any `wing_slot > 0` row).
        let wing = (1u16..6000).find(|&k| Player::wing_slot(k as i32) > 0).unwrap();
        handle_on_equip(&mut app, slot(wing), 10);
        assert!(done(&app, "HEAD_IN_THE_CLOUDS"));
        // Two armour pieces: no MATCHING_ATTIRE yet.
        app.equip.armor[0] = slot(1);
        app.equip.armor[1] = slot(2);
        handle_on_equip(&mut app, slot(3), 8);
        assert!(!done(&app, "MATCHING_ATTIRE"));
        app.equip.armor[2] = slot(3);
        handle_on_equip(&mut app, slot(3), 8);
        assert!(done(&app, "MATCHING_ATTIRE"));
        // The vanity trio rides armor[10..12] + ctx 9.
        app.equip.armor[10] = slot(244);
        app.equip.armor[11] = slot(245);
        app.equip.armor[12] = slot(246);
        handle_on_equip(&mut app, slot(246), 9);
        assert!(done(&app, "FASHION_STATEMENT"));
    }

    /// `:261-279` — DYE_HARD needs all ten dye slots (8/9 locked without
    /// expert/master) AND the five misc dyes; only ctx 12/33 run the ladder.
    #[test]
    fn dye_hard_needs_every_unlocked_dye_and_the_misc_row() {
        let mut app = app_with_player();
        for i in 0..8 {
            app.equip.dye[i] = slot(1120);
        }
        for i in 0..5 {
            app.equip.misc_dyes[i] = slot(1120);
        }
        // A non-dye context never runs the ladder.
        handle_on_equip(&mut app, slot(1120), 8);
        assert!(!done(&app, "DYE_HARD"));
        handle_on_equip(&mut app, slot(1120), 12);
        assert!(done(&app, "DYE_HARD"));
        // A missing misc dye refuses a fresh manager.
        let mut app = app_with_player();
        for i in 0..8 {
            app.equip.dye[i] = slot(1120);
        }
        handle_on_equip(&mut app, slot(1120), 33);
        assert!(!done(&app, "DYE_HARD"), "the misc dyes are empty");
    }

    /// W4-c36 — `UpdateBiomes`' reach tail (`Player.cs:16732-16776`): the
    /// wall/zone/depth ladder, the special-seed read and the death cooldown.
    #[test]
    fn the_reach_check_fires_the_wall_zone_and_depth_rows() {
        let mut world = tr_core::World::new("t".into(), 200, 800);
        world.world_surface = 200.0;
        let mut p = Player::spawn(&world);
        // A hive wall (86) under the centre fires WHERES_MY_HONEY.
        let (cx, cy) = p.center();
        let (tx, ty) = ((cx / 16.0) as i32, (cy / 16.0) as i32);
        if let Some(t) = world.tile_mut(tx, ty) {
            t.wall = 86;
        }
        let rows = reach_check(&mut p, &world, false);
        assert!(rows.contains(&("WHERES_MY_HONEY", "Reach")));
        // The spider wall (62) fires JEEPERS_CREEPERS instead.
        if let Some(t) = world.tile_mut(tx, ty) {
            t.wall = 62;
        }
        let rows = reach_check(&mut p, &world, false);
        assert!(rows.contains(&("JEEPERS_CREEPERS", "Reach")));
        assert!(!rows.contains(&("WHERES_MY_HONEY", "Reach")));
        // The underworld depth fires ITS_GETTING_HOT_IN_HERE (height-200).
        p.y = 700.0 * 16.0;
        let rows = reach_check(&mut p, &world, false);
        assert!(rows.contains(&("ITS_GETTING_HOT_IN_HERE", "Reach")));
        // A surface glowshroom fires FUNKYTOWN; a graveyard fires
        // FOUND_GRAVEYARD — the else-if chain, one arm per tick.
        p.y = 100.0 * 16.0;
        p.zones.glowshroom = true;
        let rows = reach_check(&mut p, &world, true);
        assert!(rows.contains(&("FUNKYTOWN", "Reach")));
        assert!(!rows.contains(&("FOUND_GRAVEYARD", "Reach")));
        p.zones.glowshroom = false;
        let rows = reach_check(&mut p, &world, true);
        assert!(rows.contains(&("FOUND_GRAVEYARD", "Reach")));
        // A dead tick re-arms the 100-tick cooldown and fires nothing.
        p.dead = true;
        let rows = reach_check(&mut p, &world, true);
        assert!(rows.is_empty());
        assert_eq!(p.funkytown_achievement_check_cooldown, 100);
        // The cooldown blocks FUNKYTOWN/FOUND_GRAVEYARD while it runs.
        p.dead = false;
        let rows = reach_check(&mut p, &world, true);
        assert!(!rows.contains(&("FOUND_GRAVEYARD", "Reach")));
        assert_eq!(p.funkytown_achievement_check_cooldown, 99);
        // A secret seed fires PLAY_ON_A_SPECIAL_SEED.
        world.seeds |= tr_core::seeds::SeedFlags::DRUNK;
        let rows = reach_check(&mut p, &world, false);
        assert!(rows.contains(&("PLAY_ON_A_SPECIAL_SEED", "Do")));
    }

    /// W4-c36 — `BordersMovement`'s reads (`Player.cs:24099`/`:24125`): the
    /// top border fires INTO_ORBIT, the bottom band ROCK_BOTTOM.
    #[test]
    fn the_border_reads_fire_the_orbit_and_rock_bottom_rows() {
        let world = tr_core::World::new("t".into(), 200, 800);
        let mut p = Player::spawn(&world);
        p.y = 100.0;
        let rows = reach_check(&mut p, &world, false);
        assert!(rows.contains(&("INTO_ORBIT", "Reach")));
        p.y = 800.0 * 16.0 - 640.0 - 118.0 - p.height + 1.0;
        let rows = reach_check(&mut p, &world, false);
        assert!(rows.contains(&("ROCK_BOTTOM", "Reach")));
    }

    /// W4-c36 DOC-SWEEP — `WorldGen.CheckAchievement_RealEstateAndTownSlimes`
    /// (`WorldGen.cs:5659-5677`): every ported town kind housed still leaves
    /// REAL_ESTATE_AGENT open (Truffle/Steampunker/Cyborg/Pirate have no
    /// NpcKind rows — four kinds; DD2 Bartender 550 IS ported), a homeless
    /// resident does not count, and the town-slime row never fires without
    /// the eight slime kinds. The wire is faithful but unreachable until the
    /// missing kinds land. WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_c36_the_real_estate_check_needs_the_full_roster() {
        use crate::npc::{Npc, NpcKind};
        let mut app = app_with_player();
        let ported = [
            NpcKind::Guide,
            NpcKind::Merchant,
            NpcKind::Nurse,
            NpcKind::ArmsDealer,
            NpcKind::GoblinTinkerer,
            NpcKind::Wizard,
            NpcKind::Dryad,
            NpcKind::Demolitionist,
            NpcKind::WitchDoctor,
            NpcKind::Mechanic,
            NpcKind::Angler,
            NpcKind::Clothier,
            NpcKind::Stylist,
            NpcKind::DyeTrader,
            NpcKind::Painter,
            NpcKind::PartyGirl,
            NpcKind::Golfer,
            NpcKind::TaxCollector,
            NpcKind::Zoologist,
            NpcKind::Princess,
            // W4-c36 re-probe: the Bartender IS ported (DD2Bartender, the
            // bestiary row's net_id 550) — he counts toward the roster.
            NpcKind::DD2Bartender,
        ];
        // The scan reads `kind.net_id()` (bestiary-resolved, no hardcoded
        // list) — pin the Bartender's id so the roster arm stays live when
        // the d36/d41 kind bands land.
        assert_eq!(NpcKind::DD2Bartender.net_id(), 550);
        for (i, kind) in ported.iter().enumerate() {
            let mut n = Npc::new(*kind, 32.0, 100.0, 1, 0.0);
            n.hp = n.max_hp;
            n.home = Some((2 + i as i32, 6));
            app.npcs.push(n);
        }
        check_real_estate_and_town_slimes(&mut app);
        assert!(!done(&app, "REAL_ESTATE_AGENT"), "the four missing kinds keep it open");
        assert!(!done(&app, "ALL_TOWN_SLIMES"), "no slime kinds exist");
        // A homeless resident is not housed: clearing one home changes
        // nothing observable here (the row was already open), but the
        // predicate itself is the C#'s `!homeless` read.
        app.npcs[0].home = None;
        check_real_estate_and_town_slimes(&mut app);
        assert!(!done(&app, "REAL_ESTATE_AGENT"));
        // A dead NPC does not count either (`Main.npc[i].active`).
        app.npcs[0].home = Some((2, 6));
        app.npcs[0].hp = 0;
        check_real_estate_and_town_slimes(&mut app);
        assert!(!done(&app, "REAL_ESTATE_AGENT"));
    }

    /// `:148-195` — the enter-world sweep completes the stat/equip rows off
    /// the loaded player, and the inventory fan-out feeds the pickup rows.
    #[test]
    fn the_enter_world_sweep_completes_the_loaded_rows() {
        let mut app = app_with_player();
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max_base = 200;
            p.max_hp = 500;
            p.unlocked_biome_torches = true;
        }
        app.equip.misc_equips[4] = slot(84);
        app.equip.misc_equips[3] = slot(1914);
        // A TIMBER wood in the grid rides the OnItemPickup fan-out.
        app.inventory[10] = slot(9);
        on_player_entered_world(&mut app);
        for name in [
            "STAR_POWER",
            "TOPPED_OFF",
            "HOLD_ON_TIGHT",
            "THE_CAVALRY",
            "GAIN_TORCH_GODS_FAVOR",
            "TIMBER",
        ] {
            assert!(done(&app, name), "{name}");
        }
        // A fresh 100/20 character completes none of the stat rows.
        let mut fresh = app_with_player();
        on_player_entered_world(&mut fresh);
        assert!(!done(&fresh, "STAR_POWER"));
        assert!(!done(&fresh, "TOPPED_OFF"));
    }

    /// W4-c36 NOTIFY-SYNC (1) — the TRAINING_DAY keep-one contract: g90b
    /// (`b3b82d071`, `world_sim/player.rs:1625-1627`) owns the ONLY notify-40
    /// site — `notify_progression_net(40)` on `combat_book_was_used &&
    /// combat_book_volume_two_was_used` (`Player.cs:25695-25698`). This
    /// witness pins the CONSUMER half on this branch's base: the registry
    /// row is a single `Progression(40)` condition, one notify completes
    /// it, and a second notify (the every-tick pair-stands shape g90
    /// wires) is idempotent — `notify_with` skips completed rows, so a
    /// second wire at the merge would be harmless but is still RETIRED
    /// (c36's owed armor-tail row, research/30 NOTIFY-SYNC). f12's tip has
    /// no 40 notify; f30's tip cites it. WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_c36_training_day_keeps_one_notify_forty_site() {
        use crate::achievements::CondKind;
        let mut app = app_with_player();
        // The row is exactly one Progression(40) condition — the notify
        // event id, not a flag broadcast (`AchievementInitializer.cs:414-416`).
        let row = app
            .achievements
            .list
            .iter()
            .find(|a| a.name == "TRAINING_DAY")
            .expect("the registry row");
        assert_eq!(row.conditions.len(), 1);
        assert!(matches!(row.conditions[0].kind, CondKind::ProgressionEvent(40)));
        assert!(!done(&app, "TRAINING_DAY"));
        // g90b's site shape: the both-books pair notify.
        let _ = app.notify_progression_net(40);
        assert!(done(&app, "TRAINING_DAY"));
        // The every-tick re-notify is idempotent (the completed-row skip).
        let second = app.notify_progression_net(40);
        assert_eq!(second, None, "a completed row never re-fires");
    }

    /// W4-c36 NOTIFY-SYNC (2) — NEW_DIGS's consumer half: the registry row
    /// is a single `Progression(43)` (`AchievementInitializer.cs:423-425`);
    /// f47's live seam (`a2561d6d9`, `world_sim/npcs.rs:6367`) fires
    /// `notify_progression_net(43)` when the state-25 emergence completes
    /// with the flipped variation 1 (`NPC.cs:53271-53274`). The notify
    /// completes the row; a second completion (another town NPC's
    /// emergence in a later tick) is idempotent. WRITTEN NOT RUN
    /// (code-only mode).
    #[test]
    fn w4_c36_new_digs_fires_on_progression_forty_three() {
        use crate::achievements::CondKind;
        let mut app = app_with_player();
        let row = app
            .achievements
            .list
            .iter()
            .find(|a| a.name == "NEW_DIGS")
            .expect("the registry row");
        assert_eq!(row.conditions.len(), 1);
        assert!(matches!(row.conditions[0].kind, CondKind::ProgressionEvent(43)));
        assert!(!done(&app, "NEW_DIGS"));
        // f47's post-pass latch drain shape.
        let first = app.notify_progression_net(43);
        assert_eq!(first, Some("NEW_DIGS"));
        assert!(done(&app, "NEW_DIGS"));
        // A second town NPC's emergence never re-fires the row.
        assert_eq!(app.notify_progression_net(43), None);
    }

    /// W4-c36 NOTIFY-SYNC (3) — TO_INFINITY_AND_BEYOND's consumer half:
    /// the row is a single `Flag("Do")` (`AchievementInitializer.cs:375-377`);
    /// d20b's published kill-arm hunk (research/30 NOTIFY-SYNC (3)) drains
    /// the 1020's altitude death into
    /// `complete_condition("TO_INFINITY_AND_BEYOND", "Do")`
    /// (`Projectile.cs:70857-70862` — `Kill()`'s 1020 arm, the
    /// `pos.y - height <= 640 && owner == myPlayer` gate). The named-row
    /// completion fires once; a second drone death is a no-op. WRITTEN
    /// NOT RUN (code-only mode).
    #[test]
    fn w4_c36_to_infinity_completes_on_the_do_flag() {
        use crate::achievements::CondKind;
        let mut app = app_with_player();
        let row = app
            .achievements
            .list
            .iter()
            .find(|a| a.name == "TO_INFINITY_AND_BEYOND")
            .expect("the registry row");
        assert_eq!(row.conditions.len(), 1);
        assert!(matches!(row.conditions[0].kind, CondKind::Custom("Do")));
        assert!(!done(&app, "TO_INFINITY_AND_BEYOND"));
        // The drain's completion shape (the lava-fishing pattern).
        let first = app
            .achievements
            .complete_condition("TO_INFINITY_AND_BEYOND", "Do");
        assert_eq!(first, Some("TO_INFINITY_AND_BEYOND"));
        assert!(done(&app, "TO_INFINITY_AND_BEYOND"));
        // A second altitude death never re-fires.
        assert_eq!(
            app.achievements.complete_condition("TO_INFINITY_AND_BEYOND", "Do"),
            None
        );
    }
}
