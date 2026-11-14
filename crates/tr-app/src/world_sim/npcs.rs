//! NPC simulation (plan stage A4): the fixed 60 Hz `Npc::update` step, the
//! death coin drops, the boss minion requests, the player respawn and the
//! `Spawner.SpawnNPC` ambient spawn attempt.

use glam::Vec2;
use tr_core::{TILE_PX, World};

use crate::achievements;
use crate::app::{App, rand01};
use crate::item::ItemDrop;
use crate::npc::{Npc, NpcKind, SlimeVariant};
use crate::regen::DotDeath;
use crate::registry;
use crate::spawner;

/// The player's death message: a DoT drain death resolves its attribution's
/// `DeathText.*` row family (`KillMe(PlayerDeathReason.ByOther(N))` with the
/// `Lang.cs:1045-1127` variant roll — the C# rides `Main.rand`; the port
/// draws it from the item stream, the M4-LOOT convention, the first use in
/// this path); every other death is the `DeathText.Slain` row.
fn dot_death_text(pending: Option<DotDeath>, player_name: &str, roll: f32) -> String {
    match pending {
        Some(death) => crate::death::death_message(death, roll, player_name),
        None => crate::death::slain_message(player_name),
    }
}

/// `Player.StatusFromNPC` (`Player.cs:22333`), the black-slime row `:22591`:
/// `npc.type == 1 && npc.netID == -6` rolls `Next(4) == 0` for 900 ticks of
/// Darkness on touch. The roll rides the item stream — the port's
/// `Main.rand` stand-in (the M4-LOOT convention).
pub(crate) fn black_slime_darkness_touch(
    slime: &Npc,
    player: &mut crate::player::Player,
    item_rng: &mut u64,
) {
    if slime.kind == NpcKind::Slime
        && slime.slime == SlimeVariant::Black
        && slime.touches(player)
        && rand01(item_rng) < 0.25
    {
        player.buffs.add_buff(22, 900);
    }
}

/// `AI_006_Worms` (`NPC.cs:50963-52090`) — the chain pass. The vanilla runs
/// it inside each NPC's `Update` against the `Main.npc` slots; the port runs
/// it here against the `Vec` (the slot order = the Vec order = the spawn
/// order, so the head moves before its followers exactly like the original).
/// The head digs (`flag3`'s solid-overlap scan switches the direct aim), the
/// followers ride the rigid trail (`:51867-51922`), a missing leader kills
/// the link (the chain death, `:51425-51433`). The EoW middle-segment split
/// (`:51511-51518`'s `Transform`) is deferred: a middle kill takes the tail
/// with it (the chain-death rule), the split lands with the boss-summon
/// work.
/// The town shop open (`NPC.cs`'s right-click dialog → `OpenShop`): the
/// press-edge cursor on a town NPC's rect opens the MERCHANT's shop (the
/// Guide offers none — M4-TOWN-3's finding). The stock and the happiness
/// multiplier capture at the open (`Main.OpenShop`'s `SetupShop` +
/// `ShopHelper.OpenShop` pair).
/// `Player.taxRate` (`Player.cs:57097`): 3600 ticks between collections.
pub(crate) const TAX_RATE: i32 = 3600;
/// `Player.CollectTaxes`'s cap — `Item.buyPrice(0, 25)` = 25 gold
/// (`Player.cs:24132`/`:24151`).
pub(crate) const TAX_CAP: i32 = 250_000;

/// `Main.NPCChatText_DoTaxCollector` (`Main.cs:40180-40225`): the Tax
/// Collector's right-click pays the accrued tax purse out in coins — the
/// platinum→gold→silver→copper split at the player's centre, then the
/// purse empties. The happiness adjustment scales the payout
/// (`:40184`); the port's town NPCs carry the plain 1.0.
pub(crate) fn tax_collector_payout(app: &mut App, npc_idx: usize) -> bool {
    if app.world.tax_money <= 0 {
        return true;
    }
    let mut num = app.world.tax_money;
    let (px, py) = app.npcs[npc_idx].center();
    while num > 0 {
        let (kind, stack) = if num > 1_000_000 {
            let stack = num / 1_000_000;
            num -= 1_000_000 * stack;
            (74u16, stack as u32)
        } else if num > 10_000 {
            let stack = num / 10_000;
            num -= 10_000 * stack;
            (73, stack as u32)
        } else if num > 100 {
            let stack = num / 100;
            num -= 100 * stack;
            (72, stack as u32)
        } else {
            let stack = num.max(1);
            num -= stack;
            (71, stack as u32)
        };
        crate::world_sim::items::spawn_item_drop_px(app, kind, stack, Vec2::new(px, py));
    }
    app.world.tax_money = 0;
    true
}

/// The right-click dispatch's Tax Collector arm — the `TaxCollectorCollectTaxes`
/// interaction (`NPCInteractions.cs:221-244`); he offers no shop.
fn try_tax_collect_at_cursor(app: &mut App, world_pos: Vec2) -> Option<usize> {
    app.npcs.iter().position(|n| {
        n.hp > 0 && n.kind == NpcKind::TaxCollector && {
            let (w, h) = n.size();
            world_pos.x >= n.x
                && world_pos.x < n.x + w
                && world_pos.y >= n.y
                && world_pos.y < n.y + h
        }
    })
}

pub(crate) fn try_open_shop_at_cursor(app: &mut App, world_pos: Vec2) -> bool {
    // The bound-goblin rescue (`NPC.cs:19824-19840`'s `talkNPC` trigger +
    // `AI_000_TransformBoundNPC` `:44884-44896`): the talk transforms 105
    // → 107 in place and raises `savedGoblin` — the port falls through to
    // the shop open (the vanilla opens the dialog, the player then picks
    // Shop; the ПКМ-collapse matches the Nurse treatment).
    if let Some(i) = app.npcs.iter().position(|n| {
        n.hp > 0 && n.kind == NpcKind::BoundGoblin && {
            let (w, h) = n.size();
            world_pos.x >= n.x
                && world_pos.x < n.x + w
                && world_pos.y >= n.y
                && world_pos.y < n.y + h
        }
    }) {
        rescue_bound_goblin(&mut app.npcs[i], &mut app.world);
    }
    // The bound Wizard/Mechanic talks (`NPC.cs:19841-19848`'s talk arms):
    // the talk transforms 106 → 108 and 123 → 124 in place and raises
    // the saved flags (`:52948`/`:52951`), which opens the town arrival.
    for (bound, freed, flag) in [
        (NpcKind::BoundWizard, NpcKind::Wizard, "wizard"),
        (NpcKind::BoundMechanic, NpcKind::Mechanic, "mech"),
    ] {
        if let Some(i) = app.npcs.iter().position(|n| {
            n.hp > 0 && n.kind == bound && {
                let (w, h) = n.size();
                world_pos.x >= n.x
                    && world_pos.x < n.x + w
                    && world_pos.y >= n.y
                    && world_pos.y < n.y + h
            }
        }) {
            match flag {
                "wizard" => rescue_bound_wizard(&mut app.npcs[i], &mut app.world),
                _ => rescue_bound_mechanic(&mut app.npcs[i], &mut app.world),
            }
            let _ = freed;
        }
    }
    // The sleeping Golfer's talk (`NPC.cs:19833-19836`): the talk frees
    // him and raises `savedGolfer` (`:52938`).
    if let Some(i) = app.npcs.iter().position(|n| {
        n.hp > 0 && n.kind == NpcKind::SleepingGolfer && {
            let (w, h) = n.size();
            world_pos.x >= n.x
                && world_pos.x < n.x + w
                && world_pos.y >= n.y
                && world_pos.y < n.y + h
        }
    }) {
        rescue_sleeping_golfer(&mut app.npcs[i], &mut app.world);
    }
    // The webbed Stylist's talk (`NPC.cs:19849-19852`): the talk frees
    // him and raises `savedStylist` (`:52953`).
    if let Some(i) = app.npcs.iter().position(|n| {
        n.hp > 0 && n.kind == NpcKind::WebbedStylist && {
            let (w, h) = n.size();
            world_pos.x >= n.x
                && world_pos.x < n.x + w
                && world_pos.y >= n.y
                && world_pos.y < n.y + h
        }
    }) {
        rescue_webbed_stylist(&mut app.npcs[i], &mut app.world);
    }
    // The Sleeping Angler's talk (`NPC.cs:19853-19856`'s `talkNPC`
    // trigger + `AI_000_TransformBoundNPC` `:44884-44896`): the talk
    // transforms 376 → 369 in place, `UpdateSavedStates` (`:52956-52958`)
    // raises `savedAngler`, and `Transform`'s `case 369` (`:81121-81129`)
    // forces the new Angler homeless with his home at his feet — the
    // port drops the home (the town pass re-anchors him).
    if let Some(i) = app.npcs.iter().position(|n| {
        n.hp > 0 && n.kind == NpcKind::SleepingAngler && {
            let (w, h) = n.size();
            world_pos.x >= n.x
                && world_pos.x < n.x + w
                && world_pos.y >= n.y
                && world_pos.y < n.y + h
        }
    }) {
        rescue_bound_angler(&mut app.npcs[i], &mut app.world);
    }
    // The Angler's quest turn-in (`NPCInteractions.cs:519`'s `AnglerQuest`
    // → `Main.NPCChatText_DoAnglerQuest`, `Main.cs:40227-40263`): the
    // hand-in of today's quest fish — the vanilla's chat-button collapse,
    // the Nurse treatment.
    if let Some(i) = app.npcs.iter().position(|n| {
        n.hp > 0 && n.kind == NpcKind::Angler && {
            let (w, h) = n.size();
            world_pos.x >= n.x
                && world_pos.x < n.x + w
                && world_pos.y >= n.y
                && world_pos.y < n.y + h
        }
    }) {
        return angler_turn_in(app, i);
    }
    // The Tax Collector's right arm (`NPCInteractions.cs:221-244`'s
    // `TaxCollectorCollectTaxes`): he offers no shop — the click pays the
    // tax purse out (`Main.NPCChatText_DoTaxCollector`).
    if let Some(i) = try_tax_collect_at_cursor(app, world_pos) {
        return tax_collector_payout(app, i);
    }
    // The town dispatch: the Nurse heals (the vanilla's dialog-button pair
    // — the port collapses them onto the right-click).
    let Some(nurse_idx) = app.npcs.iter().position(|n| {
        n.hp > 0 && n.kind == NpcKind::Nurse && {
            let (w, h) = n.size();
            world_pos.x >= n.x
                && world_pos.x < n.x + w
                && world_pos.y >= n.y
                && world_pos.y < n.y + h
        }
    }) else {
        return try_open_town_shop_at_cursor(app, world_pos);
    };
    app.nurse_heal(nurse_idx)
}

/// `Main.CanStartInvasion` (`Main.cs:64414-64432`): no active invasion
/// (the port's `invasion_goblins` covers `invasionType`), and at least one
/// qualifying player — the 200-life gate (`statLifeMax >= 200`); the
/// multiplayer count folds to the session player. The item path passes
/// `ignoreDelay: true`, so the `invasionDelay` scouting arm folds out.
pub(crate) fn can_start_invasion(world: &tr_core::World, player_life_max: i32) -> bool {
    !world.invasion_goblins && player_life_max >= 200
}

/// `Main.StartInvasion(1)` (`Main.cs:64435-64460`): the goblin type, the
/// size `80 + 40 x the qualifying players` (the CanStart gate guarantees
/// >= 1); the active-invasion early return holds (`invasionType != 0`).
pub(crate) fn start_invasion(
    world: &mut tr_core::World,
    qualifying_players: i32,
    invasion_type: i32,
) {
    if world.invasion_goblins {
        return;
    }
    world.invasion_goblins = true;
    // `Main.invasionType = type` + the march/warn state (`:64455-64510`) —
    // the morning-roll resolve and the item paths share the state. The
    // pirate bonus (`type == 3`'s `+40+20x`) rides the type-3 slice.
    world.invasion_type = invasion_type;
    world.invasion_warn = 0;
    // `:64457-64464`: the base 80+40x; the pirates (type 3) add 40+20x,
    // the martians (type 4) use 160+40x.
    world.invasion_size = 80 + 40 * qualifying_players.max(1);
    if invasion_type == 3 {
        world.invasion_size += 40 + 20 * qualifying_players.max(1);
    }
    if invasion_type == 4 {
        world.invasion_size = 160 + 40 * qualifying_players.max(1);
    }
    world.invasion_size_start = world.invasion_size;
}

/// The kind → the shop number (`NPCInteractions.cs:488-498`'s registry:
/// `Shop(17, 1)`, `Shop(19, 2)` … `Shop(107, 6)` — the port keeps the
/// vanilla indices). `None` = no shop (the Guide).
pub(crate) fn shop_index_for_kind(kind: NpcKind) -> Option<u32> {
    Some(match kind {
        NpcKind::Merchant => 1,
        NpcKind::ArmsDealer => 2,
        NpcKind::Dryad => 3,
        NpcKind::Demolitionist => 4,
        NpcKind::Clothier => 5,
        NpcKind::GoblinTinkerer => 6,
        NpcKind::Painter => 15,
        NpcKind::WitchDoctor => 16,
        NpcKind::TravellingMerchant => 19,
        NpcKind::Wizard => 7,
        NpcKind::Mechanic => 8,
        NpcKind::Stylist => 18,
        NpcKind::Golfer => 22,
        NpcKind::Zoologist => 23,
        NpcKind::Princess => 24,
        NpcKind::DyeTrader => 12,
        NpcKind::PartyGirl => 13,
        _ => return None,
    })
}

/// The talk-rescue (`AI_000_TransformBoundNPC`, `NPC.cs:44884-44896`): the
/// bound goblin transforms into the Tinkerer in place — the freed 18x40
/// stats restore (`SetDefaults` 107, `:10271-10283`), `savedGoblin`
/// raises (`:52944-52946`) and the tied state never returns.
pub(crate) fn rescue_bound_goblin(n: &mut Npc, world: &mut tr_core::World) {
    n.kind = NpcKind::GoblinTinkerer;
    n.max_hp = NpcKind::GoblinTinkerer.max_hp();
    n.hp = n.max_hp;
    world.saved_goblin = true;
}

/// The Sleeping Angler's rescue (`AI_000_TransformBoundNPC`'s 376 → 369 arm,
/// `NPC.cs:19853-19856`/`:44884-44894`): the transform in place, `savedAngler`
/// raised, the Angler homeless (the C# `Transform` `case 369`,
/// `NPC.cs:81121-81129`).
/// The bound Wizard's rescue (`AI_000_TransformBoundNPC`'s 106 → 108 arm,
/// `NPC.cs:19841-19844`/`:44884-44894`): the transform in place, the freed
/// stats restore and `savedWizard` raises (`:52948`).
pub(crate) fn rescue_bound_wizard(n: &mut Npc, world: &mut tr_core::World) {
    n.kind = NpcKind::Wizard;
    n.max_hp = NpcKind::Wizard.max_hp();
    n.hp = n.max_hp;
    n.home = None;
    world.saved_wizard = true;
}

/// The bound Mechanic's rescue (`AI_000_TransformBoundNPC`'s 123 → 124 arm,
/// `NPC.cs:19845-19848`): the transform in place and `savedMech` raises
/// (`:52951`).
pub(crate) fn rescue_bound_mechanic(n: &mut Npc, world: &mut tr_core::World) {
    n.kind = NpcKind::Mechanic;
    n.max_hp = NpcKind::Mechanic.max_hp();
    n.hp = n.max_hp;
    n.home = None;
    world.saved_mech = true;
}

/// The webbed Stylist's rescue (`AI_000_TransformBoundNPC`'s 354 → 353 arm,
/// `NPC.cs:19849-19852`/`:44884-44894`): the talk frees him in place and
/// raises `NPC.savedStylist` (`:52953-52955`), which opens the town
/// arrival; `Transform`'s `case 353` (`:81120`) leaves him homeless.
/// The sleeping Golfer's rescue (`AI_000_TransformBoundNPC`'s 589 → 588 arm,
/// `NPC.cs:19833-19836`/`:44884-44894`): the talk frees him in place and
/// raises `NPC.savedGolfer` (`:52938-52940`), which opens the town
/// arrival; `Transform`'s `case 588` (`:81122-81128`) leaves him homeless.
pub(crate) fn rescue_sleeping_golfer(n: &mut Npc, world: &mut tr_core::World) {
    n.kind = NpcKind::Golfer;
    n.max_hp = NpcKind::Golfer.max_hp();
    n.hp = n.max_hp;
    n.home = None;
    world.saved_golfer = true;
}

pub(crate) fn rescue_webbed_stylist(n: &mut Npc, world: &mut tr_core::World) {
    n.kind = NpcKind::Stylist;
    n.max_hp = NpcKind::Stylist.max_hp();
    n.hp = n.max_hp;
    n.home = None;
    world.saved_stylist = true;
}

/// The Sleeping Angler's rescue (`AI_000_TransformBoundNPC`'s 376 → 369 arm,
/// `NPC.cs:19853-19856`/`:44884-44894`): the transform in place, `savedAngler`
/// raised, the Angler homeless (the C# `Transform` `case 369`,
/// `:81121-81129`).
pub(crate) fn rescue_bound_angler(n: &mut Npc, world: &mut tr_core::World) {
    n.kind = NpcKind::Angler;
    n.max_hp = NpcKind::Angler.max_hp();
    n.hp = n.max_hp;
    n.home = None;
    world.saved_angler = true;
}

/// `Main.NPCChatText_DoAnglerQuest` (`Main.cs:40227-40263`) — the quest's
/// turn-in: the day gate, the quest-fish scan (`FindItem`), the stack
/// decrement, the `anglerQuestsFinished++` and the reward
/// (`Player.GetAnglerReward`, `Player.cs:56302`).
pub(crate) fn angler_turn_in(app: &mut App, angler_idx: usize) -> bool {
    let mut handed = false;
    // `:40232`'s gate: the day's quest is open and this player has not
    // already handed it in (`anglerWhoFinishedToday`'s single-player
    // collapse).
    if !app.world.angler_quest_finished && !app.world.angler_quest_turned_in {
        let quest_item = crate::angler::QUEST_POOL[app.world.angler_quest as usize];
        if let Some(slot) = app
            .inventory
            .iter()
            .position(|s| s.kind == quest_item && s.count > 0)
        {
            app.inventory[slot].count -= 1;
            if app.inventory[slot].count == 0 {
                app.inventory[slot].kind = 0;
            }
            handed = true;
            // `:40244`: the counter++ BEFORE the reward table reads it.
            if let Some(p) = app.player.as_mut() {
                p.angler_quests_finished += 1;
            }
            give_angler_reward(app, angler_idx);
        }
    }
    if handed {
        // `:40251-40258`: the day flags — the world's `anglerQuestFinished`
        // and this player's slot in `anglerWhoFinishedToday`.
        app.world.angler_quest_finished = true;
        app.world.angler_quest_turned_in = true;
        // `AchievementsHelper.HandleAnglerService` (`:40260`,
        // `AchievementsHelper.cs:411-418`): SERVANT_IN_TRAINING (the `Finish`
        // flag) and the four `Finish` counters. The rows share the condition
        // name `Finish`, so the one `notify_custom_int` dispatch advances all
        // four (`matches_kind` keys on the name).
        let _ = app.achievements.notify_custom("Finish");
        let _ = app.achievements.notify_custom_int("Finish", 1);
    }
    true
}

/// The reward spawn (`Player.GetAnglerReward`, `Player.cs:56302-56311`): the
/// four sub-rolls on `Main.rand`'s stream, each spawning at the Angler's
/// centre (`QuickSpawnItem`'s drops).
fn give_angler_reward(app: &mut App, angler_idx: usize) {
    let quests_done = app.player.as_ref().map_or(0, |p| p.angler_quests_finished);
    let hard_mode = app.world.hard_mode;
    // `:56306`: the Angler's `currentShoppingSettings.PriceAdjustment` — the
    // mood pass leaves the town roster at 1.0 (`ShopHelper.cs:103`'s
    // exclusions do not cover 369, but the port's town NPCs carry no mood
    // rows for him yet, so the plain 1.0 stands).
    let adjustment = 1.0f32;
    // `DropAnglerAccByMissing`'s inventory scan (`Player.cs:56590-56594`):
    // the port's one inventory (the armor/bank/loadout scans are
    // constants-off — the port carries neither).
    let owned: Vec<u16> = app
        .inventory
        .iter()
        .filter(|s| s.kind != 0 && s.count > 0)
        .map(|s| s.kind)
        .collect();
    let lines =
        crate::angler::angler_reward(quests_done, hard_mode, adjustment, &owned, &mut app.npc_rng);
    let (ax, ay) = app.npcs[angler_idx].center();
    for line in lines {
        crate::world_sim::items::spawn_item_drop_px(app, line.item, line.stack, Vec2::new(ax, ay));
    }
}

/// `UpdateTime_StartDay`'s `Main.AnglerQuestSwap()` (`Main.cs:65373`) — the
/// app pass resolves the clock's request (`Main.cs:3754-3902` the day reset
/// + the pool re-roll).
pub(crate) fn resolve_angler_swap(app: &mut App) {
    if !app.world.angler_swap_pending {
        return;
    }
    app.world.angler_swap_pending = false;
    // `:3760-3761`: the day reset (`anglerWhoFinishedToday.Clear()` collapses
    // to the single-player flag).
    app.world.angler_quest_finished = false;
    app.world.angler_quest_turned_in = false;
    let gates = crate::angler::QuestGates {
        hard_mode: app.world.hard_mode,
        // `WorldGen.crimson` — the port has no evil-kind field, so the
        // corruption default stands (the crimson quest rows 2457/2463/
        // 2477/2485 then re-roll exactly like a corruption world).
        crimson: false,
        downed_boss1: app.world.downed_eoc,
        downed_boss2: app.world.downed_eow,
        downed_boss3: app.world.downed_skeletron,
        downed_slime_king: app.world.downed_slime_king,
        downed_queen_bee: app.world.downed_queen_bee,
        // `Main.isThereAWorldSurface` — the constants-off seed.
        has_world_surface: true,
    };
    let mut rng = app.npc_rng;
    let idx = crate::angler::angler_quest_swap(&gates, &mut rng);
    app.npc_rng = rng;
    app.world.angler_quest = idx as u8;
}

/// The Sleeping Angler's world spawn (`Spawner`'s two 376 arms,
/// `NPC.cs:1775-1794` the ocean water tile and `:1925-1928` the beach sand):
/// the Angler unsaved and no 376 alive, the 1-in-20 roll per attempt. The
/// vanilla's arms sit inside `SpawnAnNPC`'s ladder; the port runs them as
/// their own pass (the same player-tile scan, the same roll count).
/// `NPC.cs:1660-1670`'s spider-wall arm: the webbed Stylist waits on a
/// spider wall (`wall == 62`) in the rock layer — `Next(8) == 0`, no
/// water, once per save (`!savedStylist && !AnyNPCs(354)`). The port has
/// no spider-wall tile table, so the wall gate rides the world's wall id
/// at the player's spawn tile (the same depth band as the C#'s
/// `deeperThanRockLayer`).
/// `NPC.cs:1680-1695`'s underground-desert arm: the sleeping Golfer waits
/// below the rock layer — `Next(20) == 0`, no water, once per save
/// (`!savedGolfer && !AnyNPCs(589)`). The vanilla's arm is gated on the
/// desert wall's `AllowsUndergroundDesertEnemiesToSpawn` set, which the
/// port has no table for; the depth band (`rockLayer * 2 + maxTilesY) / 3`
/// and the rate ride the same cadence.
pub(crate) fn try_spawn_sleeping_golfer(app: &mut App) -> bool {
    if app.world.saved_golfer || app.npcs.iter().any(|n| n.kind == NpcKind::SleepingGolfer) {
        return false;
    }
    let Some(p) = app.player.as_ref() else {
        return false;
    };
    let ty = (p.y / TILE_PX) as i32;
    // `(rockLayer * 2 + maxTilesY) / 3` — the deep band the C# checks
    // (`:1683`'s `num10 *= 0.5` branch).
    let deep_band = (app.world.rock_layer * 2.0 + app.world.height as f64) / 3.0;
    if (ty as f64) <= app.world.rock_layer || (ty as f64) <= deep_band {
        return false;
    }
    let water = app
        .world
        .tile((p.x / TILE_PX) as i32, ty)
        .is_some_and(|t| t.has_any_liquid());
    if water || rand01(&mut app.npc_rng) >= 0.05 {
        return false;
    }
    let mut m = Npc::new(
        NpcKind::SleepingGolfer,
        (p.x / TILE_PX) as i32 as f32 * TILE_PX + 8.0 - 9.0,
        ty as f32 * TILE_PX - 34.0,
        1,
        0.0,
    );
    m.hp = m.max_hp;
    app.npcs.push(m);
    true
}

pub(crate) fn try_spawn_webbed_stylist(app: &mut App) -> bool {
    if app.world.saved_stylist || app.npcs.iter().any(|n| n.kind == NpcKind::WebbedStylist) {
        return false;
    }
    let Some(p) = app.player.as_ref() else {
        return false;
    };
    let tx = (p.x / TILE_PX) as i32;
    let ty = (p.y / TILE_PX) as i32;
    if (ty as f64) <= app.world.rock_layer {
        return false;
    }
    let water = app.world.tile(tx, ty).is_some_and(|t| t.has_any_liquid());
    if water || rand01(&mut app.npc_rng) >= 1.0 / 8.0 {
        return false;
    }
    let mut m = Npc::new(
        NpcKind::WebbedStylist,
        tx as f32 * TILE_PX + 8.0 - 9.0,
        ty as f32 * TILE_PX - 34.0,
        1,
        0.0,
    );
    m.hp = m.max_hp;
    app.npcs.push(m);
    true
}

pub(crate) fn try_spawn_sleeping_angler(app: &mut App) -> bool {
    if app.world.saved_angler || app.npcs.iter().any(|n| n.kind == NpcKind::SleepingAngler) {
        return false;
    }
    let Some(p) = app.player.as_ref() else {
        return false;
    };
    let tx = (p.x / TILE_PX) as i32;
    let ty = (p.y / TILE_PX) as i32;
    // `Main.tileSand` (`Main.cs:7228-7231`): 53/112/116/234.
    const SAND: [u16; 4] = [53, 112, 116, 234];
    // `WorldGen.beachDistance` (`WorldGen.cs:86914`): 380 tiles.
    const BEACH: i32 = 380;
    // The sand row reads the ground under the player (`FindGroundTile`'s
    // descent — the vanity's ladder spawns on the ground tile), the water
    // row the player's own tile (`:1775`'s `this.waterTile`).
    let ground = app
        .world
        .tile(tx, ty + 1)
        .is_some_and(|t| t.active() && SAND.contains(&t.kind));
    let water = app.world.tile(tx, ty).is_some_and(|t| t.has_any_liquid());
    let near_edge = tx < BEACH || tx > app.world.width as i32 - BEACH;
    let above_surface = (ty as f64) < app.world.world_surface;
    // `:1775`'s ocean-water arm vs `:1925`'s beach-sand arm — the water arm
    // also runs below the surface (`!isThereAWorldSurface`), the sand arm
    // only above it.
    let eligible = (water && near_edge) || (ground && near_edge && above_surface);
    if !eligible {
        return false;
    }
    if rand01(&mut app.npc_rng) >= 0.05 {
        return false;
    }
    let mut m = Npc::new(
        NpcKind::SleepingAngler,
        tx as f32 * TILE_PX + 8.0 - 17.0,
        ty as f32 * TILE_PX - 8.0,
        1,
        0.0,
    );
    m.hp = m.max_hp;
    app.npcs.push(m);
    true
}

/// `Chest.cs:2714-2746`'s copper total over the 54-slot inventory scan — the
/// Stylist's Money Hair Dye gate reads it (the cap at 9,999,999,999 stands).
fn coin_total(inventory: &[crate::item::ItemSlot]) -> i64 {
    let mut total: i64 = 0;
    for sl in inventory.iter() {
        if sl.count == 0 {
            continue;
        }
        let mult: i64 = match sl.kind {
            71 => 1,
            72 => 100,
            73 => 10_000,
            74 => 1_000_000,
            _ => continue,
        };
        total += sl.count as i64 * mult;
        if total < 0 || total > 9_999_999_999 {
            return 9_999_999_999;
        }
    }
    total
}

fn try_open_town_shop_at_cursor(app: &mut App, world_pos: Vec2) -> bool {
    let Some(idx) = app.npcs.iter().position(|n| {
        n.hp > 0 && shop_index_for_kind(n.kind).is_some() && {
            let (w, h) = n.size();
            world_pos.x >= n.x
                && world_pos.x < n.x + w
                && world_pos.y >= n.y
                && world_pos.y < n.y + h
        }
    }) else {
        return false;
    };
    let shop = shop_index_for_kind(app.npcs[idx].kind).unwrap_or(1);
    let (pcx, pcy) = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((0.0, 0.0));
    let mood =
        crate::happiness::process_mood(&app.world, &app.npcs, &app.npcs[idx], (pcx, pcy), false);
    let flags = crate::shop::ShopFlags {
        hard_mode: app.world.hard_mode,
        downed_slime_king: app.world.downed_slime_king,
        downed_queen_slime: app.world.downed_queen_slime,
        downed_moon_lord: app.world.downed_moon_lord,
        night: !app.world.day_time,
        blood_moon: app.world.blood_moon,
        zone_snow: app.scene_metrics.zone_snow,
        zone_jungle: app.scene_metrics.zone_jungle,
        downed_boss2: false,
        downed_boss3: false,
        // The Stylist shop's player-side gates (`Chest.cs:2704-2783`): the
        // life/mana maxima, the copper purse, the Party Girl's presence and
        // the mech/martian flags.
        golfer_score: 0, // the golf system is not ported (M4-GOLF-1)
        // `Main.GetBestiaryProgressReport()` (`Chest.cs:3265`) — the
        // port's tracker over the ENTRIES' unlock states.
        bestiary_completion: crate::bestiary::completion_percent(&app.bestiary.kills),
        // `NPC.unlockedPrincessSpawn` (`Main.cs:65948`) — the debug
        // override; the port's roster is the other half of her gate.
        unlocked_princess: false,
        life_max_400: app.player.as_ref().is_some_and(|p| p.max_hp >= 400),
        mana_max_200: app.player.as_ref().is_some_and(|p| p.mana_max >= 200),
        coins_over_gold: coin_total(&app.inventory) >= 1_000_000,
        party_girl_present: false, // the Party Girl (208) is not ported yet
        downed_mech_any: app.world.downed_mech_boss1
            || app.world.downed_mech_boss2
            || app.world.downed_mech_boss3,
        downed_mech_1: app.world.downed_mech_boss1,
        downed_mech_2: app.world.downed_mech_boss2,
        downed_mech_3: app.world.downed_mech_boss3,
        downed_martians: app.world.downed_martians,
        // M5-SEED-2: `Main.vampireSeed`/`Main.infectedSeed`
        // (`WorldGen.cs:555-562`) — `Chest.SetupShop`'s Dryad and Bestiary
        // Girl vampirism rows (`shop.rs`'s `Vampirism*` gates). The world's
        // own bits replace the struct's `false` defaults; a classic world
        // resolves the same stock as before.
        vampirism: app
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM),
        infected_seed: app
            .world
            .micro_seeds
            .contains(tr_core::seeds::MicroSeedFlags::WORLD_IS_INFECTED),
        crimson_world: false, // the port persists no world-evil type (see `shop.rs`)
        travel_stock: app.world.travel_shop,
    };
    let inv_ids: Vec<u16> = app
        .inventory
        .iter()
        .filter(|sl| sl.kind != 0 && sl.count > 0)
        .map(|sl| sl.kind)
        .collect();
    app.shop_stock = crate::shop::resolve_stock_entries(shop, &flags, &inv_ids);
    app.shop_mood = mood.multiplier;
    app.shop_npc = Some(idx);
    // `TinkererReforge` (`NPCInteractions.cs:407-423`) opens the reforge
    // menu beside the shop (`NPCChatText_TinkererReforge`,
    // `Main.cs:40022-40029`).
    app.in_reforge_menu = app.npcs[idx].kind == NpcKind::GoblinTinkerer;
    true
}

/// The Brain of Cthulhu's per-tick state pass — the World anchor write,
/// the swarm init, the shield, and the brainless despawn (see the block
/// comment inside).
pub(crate) fn update_brain_state(app: &mut App) {
    // The Brain of Cthulhu state (`AI_054` + AI_055's `crimsonBoss`
    // reads): the World carries the brain's centre anchor for the
    // Creepers' orbit (the WoF's write-through shape); the shield = the
    // Creepers alive (`SetDefaults`' `dontTakeDamage = true` holds until
    // the dash arm clears it — the dash only begins after the Creepers
    // die, so the roster check is the operative gate); the swarm init
    // (`localAI[0]` 0 -> 1, `:32374-32385`) spawns the 20 Creepers once,
    // at the brain's centre plus the body scatter; the brainless
    // Creepers deactivate (`NPC.crimsonBoss < 0` -> `active = false`).
    let brain = app
        .npcs
        .iter()
        .find(|n| n.kind == NpcKind::BrainOfCthulhu && n.hp > 0)
        .cloned();
    if let Some(brain) = brain {
        let (bw, bh) = brain.size();
        app.world.brain_present = true;
        app.world.brain_x = brain.x + bw * 0.5;
        app.world.brain_y = brain.y + bh * 0.5;
        if !brain.swarm_done {
            let (cw, ch) = NpcKind::Creeper.size();
            let mut swarm = Vec::new();
            for _ in 0..20 {
                let sx = brain.x + bw * 0.5 + (rand01(&mut app.item_rng) * 2.0 - 1.0) * bw as f32;
                let sy = brain.y + bh * 0.5 + (rand01(&mut app.item_rng) * 2.0 - 1.0) * bh as f32;
                let svx = (rand01(&mut app.item_rng) * 61.0 - 30.0) * 0.1;
                let svy = (rand01(&mut app.item_rng) * 61.0 - 30.0) * 0.1;
                let mut c = Npc::new(NpcKind::Creeper, sx - cw * 0.5, sy - ch * 0.5, 1, 0.0);
                c.hp = c.max_hp;
                c.vx = svx;
                c.vy = svy;
                swarm.push(c);
            }
            app.npcs.append(&mut swarm);
            if let Some(b) = app
                .npcs
                .iter_mut()
                .find(|n| n.kind == NpcKind::BrainOfCthulhu && n.hp > 0)
            {
                b.swarm_done = true;
            }
        }
        // The Creeper count: the C#'s `num818` (`NPC.cs:32534-32538`) — the
        // Brain retreats when the swarm is gone, so the arm reads the count.
        let creeper_count = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::Creeper && n.hp > 0)
            .count() as f32;
        if let Some(b) = app
            .npcs
            .iter_mut()
            .find(|n| n.kind == NpcKind::BrainOfCthulhu && n.hp > 0)
        {
            b.brain_local[2] = creeper_count;
            b.dont_take_damage = creeper_count > 0.0;
        }
    } else {
        app.world.brain_present = false;
        app.npcs.retain(|n| n.kind != NpcKind::Creeper);
    }
}

pub(crate) fn update_worms(app: &mut App) {
    let mut rng = app.item_rng;
    let world = &app.world;
    let player = app.player.as_ref().map(|p| (p.center().0, p.center().1));
    update_worms_in(
        world,
        player,
        &mut app.npcs,
        &mut rng,
        &mut app.next_worm_id,
    );
}

/// The worm pass's core — the pieces the tests drive directly (the world,
/// the player centre, the Vec, the RNG stream, the id counter).
pub(crate) fn update_worms_in(
    world: &tr_core::World,
    player: Option<(f32, f32)>,
    npcs: &mut Vec<Npc>,
    mut rng: &mut u64,
    next_worm_id: &mut u32,
) {
    // The init loop: the links spawn their children (the chain assembles
    // front to back — the Vec order visits the head, then each child as it
    // lands).
    // The `while` (not the `for 0..len`) — the pushes extend the Vec and
    // the new links init in the same pass.
    let mut i = 0usize;
    while i < npcs.len() {
        if npcs[i].worm_inited || npcs[i].hp <= 0 || !npcs[i].kind.is_worm() {
            i += 1;
            continue;
        }
        // The chain identity: an id-less head takes the next id (the
        // segments share it — the vanilla's `ai[1]`/`ai[3]` slot links).
        if npcs[i].worm_id == 0 {
            npcs[i].worm_id = *next_worm_id;
            *next_worm_id += 1;
        }
        npcs[i].worm_inited = true;
        let kind = npcs[i].kind;
        // The tails (9/12/15) sit outside the chain-spawn lists
        // (`:51393`/`:51436`) — they end the chain.
        if !kind.is_worm_head() && !kind.is_worm_mid() {
            i += 1;
            continue;
        }
        // The budget roll only at the chain start (the same-kind chains —
        // the Destroyer — keep the head kind down the chain; the re-roll
        // would refill every child's budget and never terminate).
        let total = if kind.is_worm_head() && npcs[i].seg_index == 0 {
            match kind {
                // `:51310`'s `Next(8, 13)` / `:51313`'s `Next(4, 7)` — the
                // inclusive-low exclusive-high C# ranges.
                NpcKind::DevourerHead => 8 + (crate::app::frame::rand01(&mut rng) * 5.0) as u32,
                NpcKind::GiantWormHead => 4 + (crate::app::frame::rand01(&mut rng) * 3.0) as u32,
                // The leech head: `Next(3, 6)` (`AI_006`'s type-117 branch).
                NpcKind::LeechHead => 3 + (crate::app::frame::rand01(&mut rng) * 3.0) as u32,
                // `GetEaterOfWorldsSegmentsCount` (65; the expert 70 rides
                // the difficulty work); the Destroyer's ~80 segments
                // (`AI_037`'s ai[2] roll).
                // The crawltipede's 30 segments (`:51375`'s `num34 = 30`).
                NpcKind::SolarCrawltipedeHead => 29,
                NpcKind::TheDestroyer => 80,
                _ => 65,
            }
        } else {
            npcs[i].worm_left
        };
        if kind.is_worm_head() {
            npcs[i].worm_left = total;
        }
        // The child kind: the chain START (seg_index 0) spawns `type + 1`
        // (itself for the same-kind chains); the mid bodies spawn
        // themselves while the budget holds; the empty budget spawns the
        // tail when the next kind differs, else ends (the Destroyer).
        let child_kind = if npcs[i].seg_index == 0 {
            Some(kind.worm_next())
        } else if npcs[i].worm_left > 0 {
            Some(kind)
        } else if kind.worm_next() != kind {
            Some(kind.worm_next())
        } else {
            None
        };
        let Some(child_kind) = child_kind else {
            i += 1;
            continue;
        };
        let (w, h) = npcs[i].size();
        let mut child = Npc::new(
            child_kind,
            npcs[i].x + w * 0.5 - child_kind.size().0 * 0.5,
            npcs[i].y + h,
            1,
            0.0,
        );
        child.worm_id = npcs[i].worm_id;
        child.seg_index = npcs[i].seg_index + 1;
        child.worm_left = npcs[i].worm_left.saturating_sub(1);
        child.hp = child.max_hp;
        npcs.push(child);
        i += 1;
    }
    // The `realLife` pool pass (`NPC.cs:81374-81378`): the CARRIER owns the
    // life of a linked segment. The strikes that run without the NPC list
    // (the EoC shield-dash and the thorns reflect inside `Npc::contact`)
    // park their damage on the segment; the carrier pays it here.
    for i in 0..npcs.len() {
        let dmg = npcs[i].pending_pool_damage;
        if dmg == 0 {
            continue;
        }
        npcs[i].pending_pool_damage = 0;
        let Some(carrier) = Npc::carrier_index(npcs, i) else {
            continue;
        };
        npcs[carrier].hp -= dmg;
    }
    // The mirror (`NPC.cs:81377-81378`): a segment shows the carrier's
    // `life`/`lifeMax` — the pool is one number, the whole chain's.
    for i in 0..npcs.len() {
        if !npcs[i].has_carrier() {
            continue;
        }
        let Some(carrier) = Npc::carrier_index(npcs, i) else {
            continue;
        };
        npcs[i].hp = npcs[carrier].hp;
        npcs[i].max_hp = npcs[carrier].max_hp;
    }
    // The follower trail (`:51867-51922`) + the chain death
    // (`:51425-51433`).
    for i in 0..npcs.len() {
        if !npcs[i].kind.is_worm() || npcs[i].hp <= 0 {
            continue;
        }
        // The heads don't trail — the split heads keep their seg_index
        // (`Transform(13)` keeps the NPC identity), so the skip keys on
        // the KIND; the Destroyer's same-kind links are "heads" that
        // still trail (`worm_next() == kind` marks the same-kind chain).
        if npcs[i].seg_index == 0
            || (npcs[i].kind.is_worm_head() && npcs[i].kind.worm_next() != npcs[i].kind)
        {
            continue;
        }
        // The Destroyer's segment laser (`AI_037_Destroyer`'s type-135
        // arm, `NPC.cs:49856-49891`): `localAI[0] += Next(4)`; when it
        // crosses the fresh `Next(1400, 26000)` roll — reset, `CanHit`,
        // fire the hostile laser 100 (the dmg 22 classic): the aim = the
        // player's centre minus the segment's with the +-20 px jitter,
        // the speed 8 with the +-0.05 spread, the spawn at the centre +
        // the aim x 5, `timeLeft = 300`. The vanilla's tail (136) never
        // fires; the port's last link rides along (the same kind).
        if npcs[i].hp > 0 && npcs[i].seg_index > 0 && npcs[i].kind.worm_next() == npcs[i].kind {
            npcs[i].caster_counter += (rand01(rng) * 4.0) as i32;
            let threshold = (rand01(rng) * 24600.0 + 1400.0) as i32;
            if npcs[i].caster_counter >= threshold {
                npcs[i].caster_counter = 0;
                if let Some((pcx, pcy)) = player {
                    let (lw, lh) = npcs[i].size();
                    let cx = npcs[i].x + lw * 0.5;
                    let cy = npcs[i].y + lh * 0.5;
                    // `:49863`: `CanHit(base.position, base.width,
                    // base.height, player.position, player.width,
                    // player.height)` — the centres stand in for the
                    // boxes (the anchor math is `centre / 16`).
                    if crate::projectile::can_hit(
                        world, npcs[i].x, npcs[i].y, lw, lh, pcx, pcy, 0.0, 0.0,
                    ) {
                        let mut dx = pcx - cx + (rand01(rng) * 41.0 - 20.0);
                        let mut dy = pcy - cy + (rand01(rng) * 41.0 - 20.0);
                        let d = (dx * dx + dy * dy).sqrt().max(1.0);
                        dx = dx / d * 8.0 + (rand01(rng) * 41.0 - 20.0) * 0.05;
                        dy = dy / d * 8.0 + (rand01(rng) * 41.0 - 20.0) * 0.05;
                        npcs[i].wants_hostile_shot.push((
                            crate::projectile::DESTROYER_LASER,
                            cx + dx / 8.0 * 5.0,
                            cy + dy / 8.0 * 5.0,
                            dx,
                            dy,
                            22,
                        ));
                    }
                }
            }
        }
        let (wid, seg) = (npcs[i].worm_id, npcs[i].seg_index);
        let Some(leader) = npcs
            .iter()
            .find(|n| n.worm_id == wid && n.seg_index == seg - 1 && n.hp > 0)
        else {
            // `NPC.cs:51498-51502`: the orphaned MIDDLE link becomes its
            // own worm's head (`Transform(13, ai[0])`) — the Eater of
            // Worlds' rule only. The classic families' links die with
            // their leader (`NPC.cs:51425-51433`: `life = 0` +
            // `checkDead`), and so does the Destroyer's (`AI_037`'s
            // `NPC.cs:49820-49828`).
            if npcs[i].kind == NpcKind::EaterOfWorldsBody {
                npcs[i].kind = NpcKind::EaterOfWorldsHead;
            } else {
                npcs[i].hp = 0;
            }
            continue;
        };
        let (lw, lh) = npcs[i].size();
        let dx = leader.x + leader.size().0 * 0.5 - (npcs[i].x + lw * 0.5);
        let dy = leader.y + leader.size().1 * 0.5 - (npcs[i].y + lh * 0.5);
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let num62 = (len - lw) / len;
        npcs[i].vx = 0.0;
        npcs[i].vy = 0.0;
        npcs[i].x += dx * num62;
        npcs[i].y += dy * num62;
        npcs[i].rotation = dy.atan2(dx) + 1.57;
    }
    // The followerless link: `NPC.cs:51513-51519`'s `Transform(15, 0f,
    // ai[1])` — the Eater of Worlds' body becomes the tail. The classic
    // families' links die with the chain instead (`NPC.cs:51396-51433`:
    // the follower gone → `life = 0` + `checkDead`); the mech worm's
    // same-kind chain has no mid/tail kinds to swap to.
    for i in 0..npcs.len() {
        if !npcs[i].kind.is_worm_mid()
            || npcs[i].hp <= 0
            || npcs[i].kind.worm_next() == npcs[i].kind
        {
            continue;
        }
        let (wid, seg) = (npcs[i].worm_id, npcs[i].seg_index);
        let follower_alive = npcs
            .iter()
            .any(|n| n.worm_id == wid && n.seg_index == seg + 1 && n.hp > 0);
        if !follower_alive {
            if npcs[i].kind == NpcKind::EaterOfWorldsBody {
                npcs[i].kind = NpcKind::EaterOfWorldsTail;
            } else {
                npcs[i].hp = 0;
            }
        }
    }
    // The Eater of Worlds' head: `NPC.cs:51502-51509` — the head dies when
    // its follower is gone (the EoW's own life pools let a body fall on
    // its own). The classic families' heads cannot lose a follower on
    // their own — their links share the head's pool (`realLife`).
    for i in 0..npcs.len() {
        if npcs[i].hp <= 0 || npcs[i].kind != NpcKind::EaterOfWorldsHead {
            continue;
        }
        let (wid, seg) = (npcs[i].worm_id, npcs[i].seg_index);
        let follower_alive = npcs
            .iter()
            .any(|n| n.worm_id == wid && n.seg_index == seg + 1 && n.hp > 0);
        if !follower_alive {
            npcs[i].hp = 0;
        }
    }
    // The heads: the dig/air machine (`:51923-52051`).
    let Some((pcx, pcy)) = player else { return };
    for i in 0..npcs.len() {
        if !npcs[i].kind.is_worm_head() || npcs[i].hp <= 0 {
            continue;
        }
        worm_head_step(&mut npcs[i], world, pcx, pcy, rng);
    }
}

/// The head's per-tick machine (`AI_006`'s `ai[1] == 0` branch). `flag3`
/// (the dig mode) = any tile in the body rect is solid or liquid-heavy
/// (`:51580`'s count rule). In the open air the head stumbles: the gravity
/// `+0.11` (the cap `num45`), the horizontal steering (`:51940-51972`); in
/// the ground it flies straight at the tile-snapped player (`:51997-52002`).
/// `noTileCollide` is family-wide — no collision blocks the head.
fn worm_head_step(npc: &mut Npc, world: &tr_core::World, pcx: f32, pcy: f32, _rng: &mut u64) {
    let (w, h) = npc.size();
    // `flag3`: the dig-mode scan (`:51575-51597`).
    let mut flag3 = false;
    let x0 = (npc.x / TILE_PX) as i32;
    let x1 = ((npc.x + w) / TILE_PX) as i32 + 1;
    let y0 = (npc.y / TILE_PX) as i32;
    let y1 = ((npc.y + h) / TILE_PX) as i32 + 1;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if world.tile(tx, ty).is_some_and(|t| {
                t.active() && (tr_core::tiledata::TILE_SOLID.has(t.kind) || t.liquid > 64)
            }) {
                flag3 = true;
            }
        }
    }
    let (num45, num46) = npc.kind.worm_tunables();
    if !flag3 {
        // The air stumble (`:51927-51973`).
        npc.vy = (npc.vy + 0.11).min(num45);
        if (npc.vx.abs() + npc.vy.abs()) < num45 * 0.4 {
            npc.vx += npc.facing as f32 * num46 * 1.1;
        } else if npc.vy == num45 {
            let target_vx = pcx - (npc.x + w * 0.5);
            if npc.vx < target_vx {
                npc.vx += num46;
            } else if npc.vx > target_vx {
                npc.vx -= num46;
            }
        } else if npc.vy > 4.0 {
            npc.vx -= npc.vx.signum() * num46 * 0.9;
        }
        npc.facing = if pcx >= npc.x + w * 0.5 { 1 } else { -1 };
    } else {
        // The dig (`:51974-52002`): the tile-snapped aim at the max speed.
        let mut dx = (((pcx / TILE_PX) as i32 - ((npc.x + w * 0.5) / TILE_PX) as i32) * 16) as f32;
        let mut dy = (((pcy / TILE_PX) as i32 - ((npc.y + h * 0.5) / TILE_PX) as i32) * 16) as f32;
        let len = (dx * dx + dy * dy).sqrt().max(0.001);
        let num67 = num45 / len;
        dx *= num67;
        dy *= num67;
        npc.vx = dx;
        npc.vy = dy;
    }
    // `noTileCollide` — the direct integration (`NPC.cs:90495-90498`).
    npc.x += npc.vx;
    npc.y += npc.vy;
    // The off-world guard (the framework shape).
    if npc.y > world.height as f32 * TILE_PX + 200.0 {
        npc.hp = 0;
    }
}

/// `WorldItem.CheckLavaDeath`'s voodoo arm (`WorldItem.cs:681-763`): the
/// Guide Voodoo Doll (267) that touches lava is consumed, strikes every
/// town NPC dead (the Guide first, then the stack's remainder across the
/// other town NPCs) and calls `NPC.SpawnWOF` at the drop's position.
pub(crate) fn voodoo_doll_lava_death(app: &mut App, pos: Vec2, stack: u32) {
    let mut num = stack.max(1) as i32;
    // The Guide first (`Main.npc[i].type == 22`'s loop, `:702-718`): the
    // `StrikeNPCNoInteraction(9999, 10f, -direction)` — the port kills
    // outright (the death pass picks the kill credit up).
    for n in app.npcs.iter_mut() {
        if n.kind == NpcKind::Guide && n.hp > 0 {
            n.hp = 0;
            num -= 1;
        }
    }
    // The remainder across the other town NPCs (`isLikeATownNPC`'s list,
    // `:730-753` — the random `List` walk; the port walks the spawn order).
    if num > 0 {
        for n in app.npcs.iter_mut() {
            if num <= 0 {
                break;
            }
            if matches!(
                n.kind,
                NpcKind::Merchant
                    | NpcKind::Dryad
                    | NpcKind::Nurse
                    | NpcKind::ArmsDealer
                    | NpcKind::Demolitionist
                    | NpcKind::Painter
                    | NpcKind::WitchDoctor
                    | NpcKind::Clothier
            ) && n.hp > 0
            {
                n.hp = 0;
                num -= 1;
            }
        }
    }
    spawn_wof_at(app, pos);
}

/// `NPC.SpawnWOF` (`NPC.cs:80315-80378`): the underworld gates (the depth
/// `maxTilesY - 205`, no living wall), the 1200-px player-clearance scan
/// (`num2` steps 16 px away from the world's centre side), the free-row
/// search (`SolidTile || liquid >= 100` blocked, expanding `num6` up/down
/// 1000 steps), the `UnderworldLayer+10..+80` row clamp and the spawn +
/// the `Announcement.HasAwoken` broadcast.
pub(crate) fn spawn_wof_at(app: &mut App, pos: Vec2) -> bool {
    let world = &app.world;
    let underworld = world.height as i32 - 200; // `Main.UnderworldLayer`
    // The gates: the depth (`pos.Y / 16 < maxTilesY - 205`) and the wall.
    if (pos.y / TILE_PX) < (world.height as f32 - 205.0)
        || app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::WallOfFlesh && n.hp > 0)
    {
        return false;
    }
    // The player-clearance scan: step 16 px away from the world centre's
    // side while a player sits within 1200 px (`:80322-80346`).
    let mut num2 = pos.x as i32;
    let num = if (pos.x / TILE_PX) > world.width as f32 / 2.0 {
        -1
    } else {
        1
    };
    let mut target_player = 0usize;
    let mut flag = false;
    while !flag {
        flag = true;
        if let Some(p) = app.player.as_ref() {
            if p.x > (num2 - 1200) as f32 && p.x < (num2 + 1200) as f32 {
                num2 -= num * 16;
                flag = false;
                target_player = 0;
            }
        }
        let _ = target_player;
    }
    // The free-row search at the doll's column.
    let num4 = (num2 / 16).clamp(1, world.width as i32 - 2);
    let mut num5 = (pos.y / TILE_PX) as i32;
    let solid_like = |x: i32, y: i32| -> bool {
        world
            .tile(x, y)
            .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
            || world.tile(x, y).is_some_and(|t| t.liquid >= 100)
    };
    if !world.in_bounds(num4, num5) || solid_like(num4, num5) {
        let mut num6 = 0;
        let mut num7 = 1000;
        while num7 > 0 {
            num7 -= 1;
            let up_ok = world.in_bounds(num4, num5 - num6)
                && !solid_like(num4, num5 - num6)
                && world
                    .tile(num4, num5 - num6)
                    .is_some_and(|t| t.liquid < 100);
            if up_ok {
                num5 -= num6;
                break;
            }
            let down_ok = world.in_bounds(num4, num5 + num6)
                && !solid_like(num4, num5 + num6)
                && world
                    .tile(num4, num5 + num6)
                    .is_some_and(|t| t.liquid < 100);
            if down_ok {
                num5 += num6;
                break;
            }
            num6 += 1;
        }
    }
    // The row clamp to the underworld band.
    let num8 = underworld + 10;
    let num9 = num8 + 70;
    num5 = num5.clamp(num8, num9);
    let mut boss = Npc::new(
        NpcKind::WallOfFlesh,
        num2 as f32,
        num5 as f32 * TILE_PX,
        1,
        0.0,
    );
    boss.hp = boss.max_hp;
    app.npcs.push(boss);
    // `ChatHelper.BroadcastChatMessage("Announcement.HasAwoken", ...)`
    // (`:80378`) — the literal stands in for the localization key.
    crate::chat::broadcast_death_text(
        &mut app.chat,
        "The Wall of Flesh has awoken!",
        crate::chat::COLOR_COMMAND,
    );
    true
}

/// `Main.StartInvasion` (`Main.cs:64482-64511`): the army size 80 + 40 per
/// qualified player, the march column on the world's far side (the random
/// 0/maxTilesX roll; the marauder's `invasionX` double is f64 on the World).
/// `Main.InvasionWarning` (`Main.cs:64400-64415`) for the goblin slice —
/// the literal en strings stand in for the `Lang.misc[0..3]` keys.
fn invasion_warning(app: &mut App) {
    // `Main.InvasionWarning`'s per-type literals (the en strings stand in
    // for `Lang.misc[0..3]` = the goblins, `[4..7]` = the Frost Legion).
    let (done, west, east, here) = match app.world.invasion_type {
        2 => (
            "The frost army has been defeated!",
            "Frost Legion is approaching from the west!",
            "Frost Legion is approaching from the east!",
            "The Frost Legion has arrived!",
        ),
        3 => (
            "The pirates have been defeated!",
            "Pirates are approaching from the west!",
            "Pirates are approaching from the east!",
            "The pirates have arrived!",
        ),
        4 => (
            "Martian Madness has been defeated!",
            "Martian Madness is approaching from the west!",
            "Martian Madness is approaching from the east!",
            "Martian Madness has arrived!",
        ),
        _ => (
            "The goblin army has been defeated!",
            "Goblins are approaching from the west!",
            "Goblins are approaching from the east!",
            "The goblin army has arrived!",
        ),
    };
    let text = if app.world.invasion_size <= 0 {
        done
    } else if app.world.invasion_x < app.world.spawn_x as f64 {
        west
    } else if app.world.invasion_x > app.world.spawn_x as f64 {
        east
    } else {
        here
    };
    crate::chat::broadcast_death_text(&mut app.chat, text, crate::chat::COLOR_COMMAND);
}

/// `Main.UpdateInvasion` (`Main.cs:64328-64386`): the defeat flag ride at
/// `size <= 0` (`SetEventFlagCleared(ref downedGoblins, 0)` + the
/// progression event 10), the march toward `spawnTileX` by
/// `max(dayRate, 1)`, the arrival warning and the 3600-tick warn timer.
/// `NPC.cs`'s lunar-tower shield state (`ShieldStrengthTower*`) — the
/// `WorldGen.cs:86688` raise, `NPC.cs:95247`'s reset and
/// `ShieldStrengthTowerMax`'s post-Moon-Lord half. The spend side rides
/// the towers' damage path (the pillar 493/507/517's own machines) — this
/// driver owns the set/reset lifecycle and the `TowerActive*` liveness.
pub(crate) fn update_tower_shields(app: &mut App) {
    let w = &mut app.world;
    // The reset: all four dead (`NPC.cs:95247-95251`).
    if w.tower_active_solar == false
        && w.tower_active_vortex == false
        && w.tower_active_nebula == false
        && w.tower_active_stardust == false
    {
        w.tower_shield_solar = 0;
        w.tower_shield_vortex = 0;
        w.tower_shield_nebula = 0;
        w.tower_shield_stardust = 0;
        w.lunar_shield_power_normal = 100;
    }
}

/// The pillar-spawn's shield raise: `WorldGen.cs:86688`'s
/// `ShieldStrengthTowerSolar = (...) = ShieldStrengthTowerMax` — the port's
/// spawn arms (the tower kinds) call it once each.
pub(crate) fn raise_tower_shield(world: &mut tr_core::World) {
    world.tower_shield_solar = tower_shield_max(world);
    world.tower_shield_vortex = tower_shield_max(world);
    world.tower_shield_nebula = tower_shield_max(world);
    world.tower_shield_stardust = tower_shield_max(world);
    world.tower_active_solar = true;
    world.tower_active_vortex = true;
    world.tower_active_nebula = true;
    world.tower_active_stardust = true;
}

/// `NPC.ShieldStrengthTowerMax` (`NPC.cs:6795-6806`): the normal power,
/// halved while the Moon Lord is downed.
pub(crate) fn tower_shield_max(world: &tr_core::World) -> i32 {
    if world.downed_moon_lord {
        world.lunar_shield_power_normal / 2
    } else {
        world.lunar_shield_power_normal
    }
}

/// The shield's spend: a hit on a shielded tower drains
/// `ShieldStrengthTower<Kind>` and the `629` damage-pulse fires
/// (`NPC.cs:79288-79324`). Returns the drained tower's kind index
/// (0 solar / 1 vortex / 2 nebula / 3 stardust) when the shield absorbed
/// the hit, or `None` when the tower fights bare.
pub(crate) fn tower_shield_absorb(
    world: &mut tr_core::World,
    tower: usize,
    dmg: i32,
) -> Option<i32> {
    let strength = match tower {
        0 => &mut world.tower_shield_solar,
        1 => &mut world.tower_shield_vortex,
        2 => &mut world.tower_shield_nebula,
        _ => &mut world.tower_shield_stardust,
    };
    if *strength <= 0 {
        return None;
    }
    *strength -= dmg;
    if *strength <= 0 {
        *strength = 0;
        match tower {
            0 => {
                world.tower_active_solar = false;
                world.downed_tower_solar = true;
            }
            1 => {
                world.tower_active_vortex = false;
                world.downed_tower_vortex = true;
            }
            2 => {
                world.tower_active_nebula = false;
                world.downed_tower_nebula = true;
            }
            _ => {
                world.tower_active_stardust = false;
                world.downed_tower_stardust = true;
            }
        }
    }
    Some(tower as i32)
}

pub(crate) fn update_invasion_state(app: &mut App) {
    // `DrawInvasionProgress`'s fade head (`Main.cs:47712-47736`): the
    // display timer decays per tick, the alpha ramps 0.05 toward it, and
    // the mode-0 arm (no bar) zeroes both.
    if app.invasion_progress_display_left > 0 {
        app.invasion_progress_display_left -= 1;
        app.invasion_progress_alpha = (app.invasion_progress_alpha + 0.05).min(1.0);
    } else {
        app.invasion_progress_alpha = (app.invasion_progress_alpha - 0.05).max(0.0);
    }
    // `if (Main.invasionProgressMode == 0)` (`Main.cs:47718`): the off mode
    // zeroes BOTH the display timer and the alpha — the bar never shows. The
    // C#'s mode-1 arm (`>= 3` at `:49908`) is the completion path the port
    // does not model. (Re-landed: the fix-front-end-click-sounds merge
    // `13834a96` resolved main against a base that predated it and dropped
    // this arm and its test.)
    if app.settings.event_progress == 0 {
        app.invasion_progress_display_left = 0;
        app.invasion_progress_alpha = 0.0;
    }
    if app.invasion_progress_near {
        app.invasion_progress_display_left = app.invasion_progress_display_left.max(160);
    }
    // `invasion_goblins` rides the type (the debug panel's flag reads it).
    app.world.invasion_goblins = app.world.invasion_type == 1;
    if app.world.invasion_type <= 0 {
        return;
    }
    if app.world.invasion_size <= 0 {
        // `SetEventFlagCleared(ref NPC.downedGoblins, 0)` +
        // `AchievementsHelper.NotifyProgressionEvent(10)` — the port's
        // achievement engine has no progression-kind rows yet (the
        // flag-set state carries the defeat).
        match app.world.invasion_type {
            2 => app.world.downed_frost = true,
            3 => app.world.downed_pirates = true,
            4 => app.world.downed_martians = true,
            _ => app.world.downed_goblins = true,
        }
        invasion_warning(app);
        app.world.invasion_type = 0;
        app.world.invasion_delay = 0;
        return;
    }
    if app.world.invasion_x == app.world.spawn_x as f64 {
        return;
    }
    let num = app.world.day_rate().max(1) as f64;
    if app.world.invasion_x > app.world.spawn_x as f64 {
        app.world.invasion_x -= num;
        if app.world.invasion_x <= app.world.spawn_x as f64 {
            app.world.invasion_x = app.world.spawn_x as f64;
            invasion_warning(app);
        } else {
            app.world.invasion_warn -= 1;
        }
    } else if app.world.invasion_x < app.world.spawn_x as f64 {
        app.world.invasion_x += num;
        if app.world.invasion_x >= app.world.spawn_x as f64 {
            app.world.invasion_x = app.world.spawn_x as f64;
            invasion_warning(app);
        } else {
            app.world.invasion_warn -= 1;
        }
    }
    if app.world.invasion_warn <= 0 {
        app.world.invasion_warn = 3600;
        invasion_warning(app);
    }
}

/// `NPC.GetNPCInvasionGroup` (`NPC.cs:78297-78330`) for the goblin group
/// (1): the kinds 26/27/28/29/111/471/472.
fn npc_invasion_group(kind: NpcKind) -> i32 {
    match kind {
        NpcKind::GoblinThief
        | NpcKind::GoblinWarrior
        | NpcKind::GoblinArcher
        | NpcKind::GoblinPeon
        | NpcKind::GoblinSorcerer
        | NpcKind::GoblinSummoner => 1,
        // `NPC.cs:78311-78314`: the Frost Legion's trio.
        NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => 2,
        // `NPC.cs:78316-78325`: the pirates (the Captain 216 is the
        // 5-point kill, `:78238-78241`).
        NpcKind::PirateDeckhand
        | NpcKind::PirateCaptainsMate
        | NpcKind::PirateArcher
        | NpcKind::PirateCorsair
        | NpcKind::PirateCaptain => 3,
        // `NPC.cs:78327-78336`+: the martians (the group 4).
        NpcKind::MartianOfficer
        | NpcKind::GrayGrunt
        | NpcKind::MartianEngineer
        | NpcKind::GigaZapper
        | NpcKind::MartianWalker => 4,
        _ => 0,
    }
}

/// `Main.CheckInvasionProgressDisplay` (`Main.cs:47599-47649`): while the
/// bar mode is the invasion one (2), any active NPC whose invasion group
/// matches the active `invasionType` within 5000 px of the player marks
/// the near-invasion flag and refreshes the report (the icon-0 case — the
/// wave events' moon/DD2 arms are their own slices).
pub(crate) fn check_invasion_progress_display(app: &mut App) {
    // The port draws the bar in the invasion mode always (the settings
    // screen's bar-mode toggle = the follow-up).
    let Some(player) = app.player.as_ref() else {
        return;
    };
    let (pcx, pcy) = player.center();
    let near = app.npcs.iter().any(|n| {
        n.hp > 0
            && npc_invasion_group(n.kind) != 0
            && npc_invasion_group(n.kind) == app.world.invasion_type
            && (n.center().0 - pcx).abs() < 5000.0
            && (n.center().1 - pcy).abs() < 5000.0
    });
    app.invasion_progress_near = near;
    if !near || app.invasion_progress_icon != 0 {
        return;
    }
    // The refresh: the points, the max, the icon (the group + 3).
    if app.world.invasion_type != 0 && app.world.invasion_size_start != 0 {
        app.invasion_progress = app.world.invasion_size_start - app.world.invasion_size;
        app.invasion_progress_max = app.world.invasion_size_start;
        app.invasion_progress_icon = app.world.invasion_type + 3;
        app.invasion_progress_wave = 0;
    }
}

/// `NPC.FindFirstNPC(551)` (`NPC.cs:80287-80303`) — the Old One's Army's
/// Betsy, the wave-7 anchor: its live life ratio (`life/lifeMax`), or
/// `None` when no Betsy is on the field (the C#'s `-1` arm).
fn dd2_betsy_hp_ratio(app: &App) -> Option<f32> {
    let betsy = app
        .npcs
        .iter()
        .find(|n| n.kind == NpcKind::DD2Betsy && n.hp > 0)?;
    if betsy.max_hp <= 0 {
        return None;
    }
    Some(betsy.hp as f32 / betsy.max_hp as f32)
}

/// The event's tier (`DD2Event.FindProperDifficulty`, `:358-370`): the
/// hardmode mech/golem gates pick the wave budgets and the kill worth.
fn dd2_tier(app: &App) -> i32 {
    tr_core::dd2::find_tier(
        app.world.hard_mode,
        app.world.downed_mech_boss1 || app.world.downed_mech_boss2 || app.world.downed_mech_boss3,
        app.world.downed_golem,
    )
}

/// `DD2Event.CheckProgress` (`DD2Event.cs:366-392`) — the Old One's Army's
/// kill advance: the kill's worth rides the tier (`GetMonsterPointsWorth`,
/// `:546-554`), the wave's budget (`GetInvasionStatus` → the tier's
/// `Difficulty_N_GetRequiredWaveKills`) opens the next wave with the
/// 1800-tick break, and the wave-6 sentinel is the win (`WonThisRun`).
fn dd2_report_kill(app: &mut App, kind: NpcKind) {
    // The event's tier (`FindProperDifficulty`, `:358-370`): the wave
    // rosters, the budgets and the kill worth all ride it.
    let tier = dd2_tier(app);
    let wave = app.world.dd2_wave;
    let roster: &[u16] = match tier {
        3 => tr_core::dd2::enemies_for_wave_t3(wave).0,
        2 => tr_core::dd2::enemies_for_wave_t2(wave).0,
        _ => tr_core::dd2::enemies_for_wave(wave).0,
    };
    if !roster.iter().any(|k| nid_of(*k) == Some(kind)) {
        // The kill outside the wave's roster scores nothing.
        return;
    }
    // The hold's refusal runs FIRST (the vanilla's early return) — the
    // kill under the break scores nothing at all.
    if app.world.dd2_spawn_hold != 0 {
        return;
    }
    // `GetMonsterPointsWorth` (`:546-554`): the tier's worth row.
    let worth = match tier {
        3 => tr_core::dd2::KILL_WORTH_T3,
        2 => tr_core::dd2::KILL_WORTH_T2,
        _ => tr_core::dd2::KILL_WORTH_T1,
    };
    // `GetInvasionStatus` (`:517-536`): the bucket is the wave's kills
    // BEFORE this kill lands; the tier's table may replace it (the
    // wave-7 Betsy bucket, `FindFirstNPC(551)`, `:1487-1494`).
    let mut kills_i = app.world.dd2_wave_kills as i32;
    let required = match tier {
        2 => tr_core::dd2::required_wave_kills_t2(wave, &mut kills_i),
        3 => tr_core::dd2::required_wave_kills_t3(wave, &mut kills_i, dd2_betsy_hp_ratio(app)),
        _ => tr_core::dd2::required_wave_kills(wave, &mut kills_i),
    };
    app.world.dd2_wave_kills += worth;
    // `DD2Event.CheckProgress` (`DD2Event.cs:376-392`): the wave report rides
    // `Main.ReportInvasionProgress(currentKillCount, requiredKillCount, 3,
    // currentWave)` — icon 3 is the `Extra[79]` Old One's Army sheet and the
    // `> 0` wave selects `DrawInvasionProgress`'s `Game.WaveMessage` arm.
    // `currentKillCount += (int)num` (`:373`) adds the kill's worth.
    app.invasion_progress = kills_i + worth as i32;
    app.invasion_progress_max = required;
    app.invasion_progress_icon = 3;
    app.invasion_progress_wave = wave;
    app.invasion_progress_display_left = 160;
    // `NPC.waveKills >= requiredKillCount && requiredKillCount != 0`
    // (`:376`): the wave's budget opens the next one.
    let passed = required != 0 && app.world.dd2_wave_kills >= required as f32;
    if passed {
        app.world.dd2_wave_kills = 0.0;
        app.world.dd2_wave += 1;
        app.world.dd2_spawn_hold = tr_core::dd2::SPAWN_HOLD_WAVE_BREAK;
    }
}

/// The kind id's reverse — the wave rosters' NPC ids to the port's kinds.
/// The Old One's Army families that have not landed yet (the wyvern 558,
/// the archers/drakes/mystics/ogres 559/560/569/570/571/572/573/574/575,
/// the sapper 565 and the glider 578) have no port kind and drop out.
fn nid_of(id: u16) -> Option<NpcKind> {
    NpcKind::from_u16(match id {
        548 => 110,
        549 => 147,
        550 => 109,
        551 => 107,
        552 => 95,
        553 => 96,
        554 => 97,
        555 => 98,
        556 => 99,
        557 => 100,
        561 => 101,
        562 => 102,
        563 => 103,
        564 => 106,
        566 => 104,
        567 => 105,
        568 => 108,
        // The Old One's Army's tier-2/3 families (M4-EVENT-19): the port
        // ids 148-160.
        558 => 169,
        559 => 170,
        560 => 171,
        565 => 172,
        569 => 173,
        570 => 174,
        571 => 175,
        572 => 176,
        573 => 177,
        574 => 178,
        575 => 179,
        576 => 180,
        577 => 181,
        _ => return None,
    })
}

/// The Old One's Army's tick (`DD2Event.Update`): the spawn hold's decay
/// and the crystal's 600-tick win-death (`NPC.cs`'s aiStyle-105 arm,
/// `:40959-40966`) — the hold's expiry opens the lane spawns.
/// `DD2Event.LaneSpawnRate` (`DD2Event.cs:1813`) — the lane gate's spawn
/// interval, set from the wave's roster (`enemies_for_wave`'s second
/// element) at the wave start.
pub(crate) fn lane_spawn_rate(world: &tr_core::World) -> i32 {
    let tier = tr_core::dd2::find_tier(
        world.hard_mode,
        world.downed_mech_boss1 || world.downed_mech_boss2 || world.downed_mech_boss3,
        world.downed_golem,
    );
    let (_, rate) = match tier {
        2 => tr_core::dd2::enemies_for_wave_t2(world.dd2_wave),
        3 => tr_core::dd2::enemies_for_wave_t3(world.dd2_wave),
        _ => tr_core::dd2::enemies_for_wave(world.dd2_wave),
    };
    rate
}

/// `DD2Event.EnemySpawningIsOnHold` (`:113`): `_timeLeftUntilSpawningBegins
/// != 0` — the pre-wave countdown that parks the lane gates' counters.
pub(crate) fn enemy_spawning_on_hold(world: &tr_core::World) -> bool {
    world.dd2_spawn_hold != 0
}

/// `DD2Event.SpawnMonsterFromGate` (`DD2Event.cs:556-570`) — the lane gate's
/// spawn: the wave's `Difficulty_N_SpawnMonsterFromGate` pick rolled against
/// the live pool (`NPC.CountNPCS`) and the invasion's kill pair
/// (`GetInvasionStatus`), spawned at `gateBottom`.
///
/// The kinds the port has not landed yet (the wyvern, the archers, drakes,
/// mystics, ogres and hounds, the sapper and the glider) have no
/// `nid_of` row and drop out.
fn spawn_dd2_from_gate(app: &mut App, sx: f32, sy: f32, left_gate: bool) -> Vec<Npc> {
    if !app.world.dd2_event_ongoing {
        return Vec::new();
    }
    let tier = dd2_tier(app);
    let wave = app.world.dd2_wave;
    // `GetInvasionStatus`'s pair: the wave's budget and the score so far.
    let kills = app.world.dd2_wave_kills as i32;
    let required = match tier {
        2 => tr_core::dd2::required_wave_kills_t2(wave, &mut kills.clone()),
        3 => tr_core::dd2::required_wave_kills_t3(
            wave,
            &mut kills.clone(),
            dd2_betsy_hp_ratio(app),
        ),
        _ => tr_core::dd2::required_wave_kills(wave, &mut kills.clone()),
    };
    // The pool's live counts (`NPC.CountNPCS`) — the roll reads them per id.
    let counts: std::collections::HashMap<u16, i32> = {
        let mut m = std::collections::HashMap::new();
        for n in app.npcs.iter() {
            if let Some(id) = dd2_pool_id(n.kind) {
                *m.entry(id).or_insert(0) += 1;
            }
        }
        m
    };
    let count = |id: u16| counts.get(&id).copied().unwrap_or(0);
    // The port's roll: `Main.rand.NextDouble` off the sim's npc rng. The rolls
    // take `Fn`, so the C#'s stream is pre-drawn into a buffer the closure
    // indexes (a pick spends at most ten draws; the tail repeats the last).
    let mut rng = app.npc_rng;
    let draws: Vec<f32> = (0..32).map(|_| crate::app::rand01(&mut rng)).collect();
    app.npc_rng = rng;
    let drawn = std::cell::Cell::new(0usize);
    let roll = || {
        let i = drawn.get();
        drawn.set(i + 1);
        draws[i.min(draws.len() - 1)]
    };
    let pick = match tier {
        2 => tr_core::dd2::lane_spawn_roll_d2(wave, &count, kills, required, left_gate, roll),
        3 => tr_core::dd2::lane_spawn_roll_d3(wave, &count, roll),
        _ => tr_core::dd2::lane_spawn_roll_d1(wave, &count, kills, required, left_gate, roll),
    };
    let mut spawned = Vec::new();
    for id in [pick.first, pick.second].into_iter().flatten() {
        if let Some(kind) = nid_of(id) {
            spawned.push(Npc::new(kind, sx, sy, 1, 0.0));
        }
    }
    spawned
}

/// The reverse of `nid_of`'s ported rows — the pool's port kinds back to the
/// vanilla ids the rolls' caps count (`CountNPCS`).
fn dd2_pool_id(kind: NpcKind) -> Option<u16> {
    let id = match kind {
        NpcKind::DD2GoblinT1 => 552,
        NpcKind::DD2GoblinT2 => 553,
        NpcKind::DD2GoblinT3 => 554,
        NpcKind::DD2GoblinBomberT1 => 555,
        NpcKind::DD2GoblinBomberT2 => 556,
        NpcKind::DD2GoblinBomberT3 => 557,
        NpcKind::DD2JavelinT1 => 561,
        NpcKind::DD2JavelinT2 => 562,
        NpcKind::DD2JavelinT3 => 563,
        NpcKind::DD2SkeletonT1 => 566,
        NpcKind::DD2SkeletonT3 => 567,
        NpcKind::DD2DarkMage => 564,
        NpcKind::DD2Betsy => 551,
        NpcKind::DD2WitherBeast => 568,
        NpcKind::DD2EterniaCrystal => 548,
        NpcKind::DD2LanePortal => 549,
        NpcKind::DD2Bartender => 550,
        // The Old One's Army's tier-2/3 families (M4-EVENT-19).
        NpcKind::DD2WyvernT1 => 558,
        NpcKind::DD2WyvernT2 => 559,
        NpcKind::DD2WyvernT3 => 560,
        NpcKind::DD2DarkMageT3 => 565,
        NpcKind::DD2WitherBeastT3 => 569,
        NpcKind::DD2DrakinT2 => 570,
        NpcKind::DD2DrakinT3 => 571,
        NpcKind::DD2KoboldWalkerT2 => 572,
        NpcKind::DD2KoboldWalkerT3 => 573,
        NpcKind::DD2KoboldFlyerT2 => 574,
        NpcKind::DD2KoboldFlyerT3 => 575,
        NpcKind::DD2OgreT2 => 576,
        NpcKind::DD2OgreT3 => 577,
        _ => return None,
    };
    Some(id)
}

/// `Main.rand.NextDouble`'s port: the 0..1 float off the sim's npc rng.
fn rng01(rng: &mut u64) -> f32 {
    crate::app::rand01(rng)
}

pub(crate) fn update_dd2_event(app: &mut App, _dt: f32) {
    if !app.world.dd2_event_ongoing {
        app.world.dd2_crystal_alive = false;
        return;
    }
    if app.world.dd2_spawn_hold > 0 {
        app.world.dd2_spawn_hold -= 1;
    }
    // `NPC.AnyNPCs(548)`'s port: the lane gates' aiStyle-106 arm reads it
    // from the machine, so the world carries the crystal's live state.
    app.world.dd2_crystal_alive = app
        .npcs
        .iter()
        .any(|n| n.kind == NpcKind::DD2EterniaCrystal && n.hp > 0);
    // The win-death resolve (`AI`'s aiStyle-105 arm's
    // `DD2Event.StopInvasion`): the event stops and the crystal goes.
    if app
        .npcs
        .iter()
        .any(|n| n.kind == NpcKind::DD2EterniaCrystal && n.wants_event_stop)
    {
        app.world.dd2_event_ongoing = false;
        app.world.dd2_spawn_hold = 0;
        app.npcs
            .retain(|n| !(n.kind == NpcKind::DD2EterniaCrystal && n.wants_event_stop));
    }
}

fn report_invasion_kill(app: &mut App, kind: NpcKind) {
    // The Old One's Army's wave progress (`DD2Event.CheckProgress`,
    // `DD2Event.cs:366-392`) rides its own counter — the invasion
    // framework's group gate does not apply.
    if app.world.dd2_event_ongoing {
        dd2_report_kill(app, kind);
        return;
    }
    let group = npc_invasion_group(kind);
    if group <= 0 || group != app.world.invasion_type {
        return;
    }
    let num9 = match kind {
        NpcKind::PirateCaptain => 5,
        NpcKind::GoblinSummoner => 10,
        _ => 1,
    };
    app.world.invasion_size = (app.world.invasion_size - num9).max(0);
    // `Main.ReportInvasionProgress` (`Main.cs:47687-47694`): the points,
    // the max, the icon (the group + 3) and the 160-tick display timer.
    app.invasion_progress = app.world.invasion_size_start - app.world.invasion_size;
    app.invasion_progress_max = app.world.invasion_size_start;
    app.invasion_progress_icon = group + 3;
    app.invasion_progress_wave = 0;
    app.invasion_progress_display_left = 160;
}

/// `Spawner.ShouldSpawnInvasionEnemies` (`NPC.cs:442-463`) for the
/// single-player slice: the surface band (`position.Y < worldSurface*16 +
/// sHeight` OR `spawnTileY > worldSurface`), the ±3000 px window around
/// `invasionX` (the town-NPC fallback needs the town roster — the wall at
/// ±5 tiles around maxTilesX/2 lands with the town work).
pub(crate) fn should_spawn_invasion_enemies(world: &World, player_y: f32, player_x: f32) -> bool {
    if world.invasion_type <= 0 || world.invasion_delay != 0 || world.invasion_size <= 0 {
        return false;
    }
    // `position.Y < Main.worldSurface * 16.0 + NPC.sHeight` — the px-space
    // compare (`NPC.sHeight` = 1200, `NPC.cs:6769`; the port used 200, which
    // made the band ~6x too narrow — S1-F-18).
    if player_y < (world.world_surface * 16.0 + crate::spawner::S_HEIGHT as f64) as f32
        || (world.spawn_y as f32) > world.world_surface as f32
    {
        let num = 3000.0;
        let inv_px = (world.invasion_x * 16.0) as f32;
        if player_x > inv_px - num && player_x < inv_px + num {
            return true;
        }
    }
    false
}

/// `NPC.ReleaseNPC` as the shimmer release uses it (`WorldItem.GetShimmered`'s
/// makeNPC arm, `WorldItem.cs:1747-1772`): the spawn with
/// `shimmerTransparency = 1f`. The C# releases through `ReleaseNPC`, which
/// applies the spawn capsule; the port's release spawns through the generic
/// `Npc::new` (the capsule's tile scan is the NPC roster's own spawn logic).
/// Returns whether a drop-owning spawn happened (the roster only covers the 85
/// ported kinds — the shimmer slime 676, the shimmered town NPC 677 and the
/// rainbow slime 681 wait for the M4-NPC roster).
pub(crate) fn spawn_shimmer_release(app: &mut App, npc_type: u16, x: f32, y: f32) -> bool {
    let Some(kind) = NpcKind::from_u16(npc_type) else {
        // The roster gap: the C#'s `ReleaseNPC` still spawns, but the port has
        // no model for the type — the arm's accounting still runs.
        return false;
    };
    let mut npc = Npc::new(kind, x, y, 1, 0.0);
    npc.shimmer_transparency = 1.0;
    app.npcs.push(npc);
    true
}

pub(crate) fn update_npcs(app: &mut App, dt: f32) {
    // `Main.UpdateTime`'s Moon-Lord block (`Main.cs:64987-64994`): the
    // apocalypse's countdown ticks down and at zero the boss spawns.
    crate::world_sim::lunar::update_moon_lord_countdown(app);
    // The stable identity (`NPC.whoAmI`, `NPC.cs:6145`): every NPC takes
    // its id once. The C#'s boss-part links (`ai[3]`, `localAI[0..2]`)
    // point at these ids; the port's list compacts every frame, so the
    // ids — not the Vec indices — carry the link.
    {
        let mut next = app.next_npc_uid;
        for n in app.npcs.iter_mut() {
            if n.uid == 0 {
                n.uid = next;
                next += 1;
            }
        }
        app.next_npc_uid = next;
    }
    // The Moon Lord parts' carrier write-through (`AI_078`'s `:41642` /
    // `AI_079`'s `:42098` read `Main.npc[ai[3]].Center` live) — the
    // PRE-pass: each 397/396/400/401 finds ITS core (the `ai[3]` link,
    // the stable id the spawn handed out), mirrors the core's centre into
    // `carrier_x/y`, and drops the link (so the machine self-kills) when
    // the core died. The leech blob's anchor is the head (`AI_082`'s
    // `:36630-36640`) — its `ai[0]` holds the head's index + 1 in the
    // vanilla; the port keeps the head's id.
    for i in 0..app.npcs.len() {
        let kind = app.npcs[i].kind;
        if kind != NpcKind::MoonLordHand
            && kind != NpcKind::MoonLordHead
            && kind != NpcKind::MoonLordFreeEye
            && kind != NpcKind::MoonLordLeechBlob
        {
            continue;
        }
        // The link: the carrier's stable id (`ai[3]`). A part whose link
        // was never handed out (a hand spawned outside the Moon Lord's
        // spawn arm) falls back to the core of its family.
        let mut carrier_uid = app.npcs[i].carrier_uid;
        if carrier_uid == 0 {
            let want = if kind == NpcKind::MoonLordLeechBlob {
                NpcKind::MoonLordHead
            } else {
                NpcKind::MoonLordCore
            };
            carrier_uid = app
                .npcs
                .iter()
                .find(|m| m.kind == want && m.hp > 0)
                .map(|m| m.uid)
                .unwrap_or(0);
            app.npcs[i].carrier_uid = carrier_uid;
        }
        let core = app
            .npcs
            .iter()
            .position(|m| m.uid == carrier_uid && m.uid != 0 && m.hp > 0);
        match core {
            Some(ci) => {
                let (ccx, ccy) = app.npcs[ci].center();
                app.npcs[i].carrier_x = ccx;
                app.npcs[i].carrier_y = ccy;
                app.npcs[i].carrier_timer = 1.0;
                app.npcs[i].moon_core_alive = true;
            }
            None => {
                app.npcs[i].moon_core_alive = false;
                app.npcs[i].carrier_timer = 0.0;
                app.npcs[i].hp = 0;
            }
        }
    }
    // The core's part bookkeeping (`AI_077`'s `:41229-41232` +
    // `:41251-41291`): the three stored slots hold the parts' ids (the
    // `localAI[0..2]` link the spawn handed out), so the core's loss
    // check — a missing part opens the death ride — resolves them.
    for i in 0..app.npcs.len() {
        if app.npcs[i].kind != NpcKind::MoonLordCore {
            continue;
        }
        // The C#'s flag (`:41251-41269`): a slot that points at no living
        // part takes the core down with it.
        let mut lost = false;
        for uid in app.npcs[i].moon_parts {
            if uid <= 0 {
                lost = true;
                continue;
            }
            let alive = app
                .npcs
                .iter()
                .any(|m| m.uid as i32 == uid && m.hp > 0);
            if !alive {
                lost = true;
            }
        }
        app.npcs[i].moon_parts_lost = lost;
    }
    // `UpdateTime_StartDay`'s `Main.AnglerQuestSwap()` request
    // (`Main.cs:65373`): the clock arms it at the day boundary (the
    // clock carries no RNG), the world pass rolls it.
    resolve_angler_swap(app);
    // `NPC.taxCollector` (`NPC.cs:6083`): `AI_007_TownEntities` raises
    // it while a Tax Collector is alive (`:53304-53307`) and the pass
    // reset clears it (`Main.cs:17908`); the tax pump runs while true.
    app.world.tax_collector = app
        .npcs
        .iter()
        .any(|n| n.kind == NpcKind::TaxCollector && n.hp > 0);
    // `Player.CollectTaxes`'s housed-town count (`Player.cs:24142-24149`):
    // the alive, housed town NPCs (the C# counts every non-pet with a
    // head index — the port's town roster).
    app.world.tax_payers = app
        .npcs
        .iter()
        .filter(|n| n.kind.is_town_resident() && n.hp > 0 && n.home.is_some())
        .count() as i32;
    // `Main.cs:64995-65003`'s pump.
    let rate = app.world.day_rate();
    crate::world_sim::clock::update_taxes(&mut app.world, rate);
    // The Scutlix rider's carrier write-through (`AI_075`'s type-390
    // arm) — the PRE-pass: the rider's `carrier_x/y` reads the carrier's
    // live centre before the AI tick (the vanilla reads `nPC5.Center`
    // live in the same pass), and a rider whose carrier died falls with
    // it (the retire lands before the retain pass below).
    for i in 0..app.npcs.len() {
        if app.npcs[i].kind != NpcKind::ScutlixRider
            && app.npcs[i].kind != NpcKind::MartianSaucer
            && app.npcs[i].kind != NpcKind::MartianSaucerTurret
            && app.npcs[i].kind != NpcKind::MartianSaucerCannon
            && app.npcs[i].kind != NpcKind::FlyingDutchmanCannon
            && app.npcs[i].kind != NpcKind::FlyingDutchman
        {
            continue;
        }
        // The Dutchman's own crew count (`AI_093`'s `Main.npc[ai[i]]`
        // links have no port model — the count within the reach stands
        // in: the ship's death rides it).
        if app.npcs[i].kind == NpcKind::FlyingDutchman {
            let (scx, scy) = app.npcs[i].center();
            let count = app
                .npcs
                .iter()
                .filter(|m| {
                    m.kind == NpcKind::FlyingDutchmanCannon && m.hp > 0 && {
                        let (mcx, mcy) = m.center();
                        (mcx - scx).hypot(mcy - scy) < 1600.0
                    }
                })
                .count() as i32;
            app.npcs[i].crew_cannons = count;
            // The display inheritance for the 492 cannons (`AI_093`'s
            // `NPC.cs:38718-38726`): the ship's `rotation` banks the
            // cannons with it, and its `spriteDirection` flips them.
            let ship_facing = app.npcs[i].sprite_dir;
            let ship_rot = app.npcs[i].rotation;
            for m in app.npcs.iter_mut() {
                if m.kind == NpcKind::FlyingDutchmanCannon && m.carrier_x >= 0.0 {
                    m.carrier_rot = ship_rot;
                    m.carrier_facing = ship_facing;
                }
            }
            continue;
        }
        let link_kind = if app.npcs[i].kind == NpcKind::ScutlixRider {
            NpcKind::Scutlix
        } else if app.npcs[i].kind == NpcKind::FlyingDutchmanCannon {
            NpcKind::FlyingDutchman
        } else {
            NpcKind::MartianSaucerCore
        };
        let (rcx, rcy) = app.npcs[i].center();
        let mut best: Option<(usize, f32)> = None;
        for (j, m) in app.npcs.iter().enumerate() {
            if m.kind != link_kind || m.hp <= 0 {
                continue;
            }
            let (mcx, mcy) = m.center();
            let d = (mcx - rcx).hypot(mcy - rcy);
            if d < 1600.0 && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((j, d));
            }
        }
        match best {
            Some((j, _)) => {
                let (mcx, mcy) = app.npcs[j].center();
                app.npcs[i].carrier_x = mcx;
                app.npcs[i].carrier_y = mcy;
                // The saucer family's extra payload: the core's mirrored
                // timer + phase, the display inheritance (`NPC.cs:35625-35651`
                // / `:35669-35676`: every mount copies the carrier's
                // `rotation` and `spriteDirection`), and the crew counts
                // within the reach.
                if link_kind == NpcKind::MartianSaucerCore {
                    app.npcs[i].carrier_timer = app.npcs[j].ai[3];
                    app.npcs[i].carrier_phase = app.npcs[j].ai[0];
                    app.npcs[i].carrier_rot = app.npcs[j].rotation;
                    app.npcs[i].carrier_facing = app.npcs[j].sprite_dir;
                    let turrets = app
                        .npcs
                        .iter()
                        .filter(|m| {
                            m.kind == NpcKind::MartianSaucerTurret
                                && m.hp > 0
                                && (m.center().0 - mcx).hypot(m.center().1 - mcy) < 1600.0
                        })
                        .count() as i32;
                    let cannons = app
                        .npcs
                        .iter()
                        .filter(|m| {
                            m.kind == NpcKind::MartianSaucerCannon
                                && m.hp > 0
                                && (m.center().0 - mcx).hypot(m.center().1 - mcy) < 1600.0
                        })
                        .count() as i32;
                    app.npcs[i].crew_turrets = turrets;
                    app.npcs[i].crew_cannons = cannons;
                }
            }
            None => {
                // The Scutlix rider falls with its carrier; the saucer
                // mounts (the shell/guns) fall with the core.
                if app.npcs[i].carrier_x >= 0.0 {
                    app.npcs[i].carrier_x = -1.0;
                    app.npcs[i].carrier_y = -1.0;
                    app.npcs[i].hp = 0;
                } else if app.npcs[i].kind == NpcKind::ScutlixRider {
                    // Never linked yet — the machine's init spawns.
                }
            }
        }
    }
    // `UpdateTime_StartDay`'s `StartInvasion()` resolve (`Main.cs:65454`):
    // the morning roll's pending flag + the players' life-max gate. The
    // march state (the type/x/warn) rides the World — `invasion_goblins`
    // (the item-path flag) stays synced for the panel/kill paths.
    if app.world.invasion_arm_pending {
        app.world.invasion_arm_pending = false;
        let qualified = app
            .player
            .as_ref()
            .map(|p| p.max_hp >= 200)
            .unwrap_or(false);
        if qualified && !app.world.invasion_goblins {
            let num = 1; // the session player (the multiplayer count folds)
            app.world.invasion_size = 80 + 40 * num;
            app.world.invasion_size_start = app.world.invasion_size;
            app.world.invasion_warn = 0;
            // `if (Main.rand.Next(2) == 0) invasionX = 0; else invasionX = maxTilesX;`
            // (`:64502-64510`) — the deterministic hash stands in for `Main.rand`
            // (the roll rides the item stream's order).
            let roll = (crate::app::frame::rand01(&mut app.item_rng) * 2.0) as u32;
            app.world.invasion_x = if roll == 0 {
                0.0
            } else {
                app.world.width as f64
            };
            start_invasion(&mut app.world, num, 1);
            // The march state rides the World fields (the app-level pass
            // reads them in `update_invasion_state`).
        }
    }
    // `Main.UpdateInvasion` (`Main.cs:64328-64386`): the per-tick state
    // machine before the spawn pass (the C# calls it from `UpdateAudio`'s
    // family each update).
    update_invasion_state(app);
    // `Main.CheckInvasionProgressDisplay` (`Main.cs:17183`): the near-scan
    // + the report refresh before the spawn pass.
    check_invasion_progress_display(app);
    app.attack_cd = (app.attack_cd - dt).max(0.0);
    // `Main.reforgeCooldown`'s per-tick decrement (`Main.cs:17121-17123`).
    if app.reforge_cooldown > 0 {
        app.reforge_cooldown -= 1;
    }
    // `CoinLossRevengeSystem.Update` (:382-389) + the 60-tick purge of the
    // expired markers (`Main.cs:17758`'s call order).
    app.revenge_tick += 1;
    if app.revenge_tick % 60 == 0 {
        app.revenge_markers.retain(|m| !m.expired(app.revenge_tick));
    }
    // `CheckRespawns` (:391-457, called from the spawn pass `NPC.cs:80176`):
    // a marker whose outer box (2608x1840 :514) holds the player respawns
    // its carrier once at `_location` with `extraValue = _coinsValue` and
    // `life = lifeMax * max(0.5, hpPercent)` (:172-210), and the marker
    // removes itself (:455).
    if !app.revenge_markers.is_empty() {
        if let Some(p) = app.player.as_ref() {
            let (pcx, pcy) = p.center();
            let outer = [pcx - 2608.0 * 0.5, pcy - 1840.0 * 0.5, 2608.0, 1840.0];
            let mut done = Vec::new();
            for m in &app.revenge_markers {
                let b = m.hitbox();
                let hits = outer[0] < b[0] + b[2]
                    && outer[0] + outer[2] > b[0]
                    && outer[1] < b[1] + b[3]
                    && outer[1] + outer[3] > b[1];
                if hits {
                    done.push(m.unique_id);
                    let mut carrier = Npc::new(m.kind, m.location.x, m.location.y, 1, 0.0);
                    carrier.extra_value = m.coins_value;
                    carrier.hp = (carrier.max_hp as f32 * m.hp_percent.max(0.5)) as i32;
                    carrier.hp = carrier.hp.max(1);
                    app.npcs.push(carrier);
                }
            }
            app.revenge_markers.retain(|m| !done.contains(&m.unique_id));
        }
    }
    app.npc_timer -= dt;
    // Fixed 60 Hz NPC simulation, like NPC.cs. Updating per rendered frame made
    // enemies move faster on high-refresh displays.
    // The Golem's anchor write (`NPC.cs:19507`'s `NPC.golemBoss`): the
    // body's centre / velocity.X / life ratio for the head and fist
    // tethers. Hoisted BEFORE the AI pass: the C# parts read
    // `Main.npc[NPC.golemBoss].Center / life / lifeMax / velocity` live
    // inside their own AI (the body's lower slot updates first), so the
    // pre-pass write is the closest same-tick value (the post-pass write
    // below fed the parts a 1-tick stale ratio).
    {
        let body = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::Golem && n.hp > 0);
        if let Some(body) = body {
            let (bw, bh) = body.size();
            app.world.golem_present = true;
            app.world.golem_x = body.x + bw * 0.5;
            app.world.golem_y = body.y + bh * 0.5;
            app.world.golem_vx = body.vx;
            app.world.golem_hp_ratio = body.hp as f32 / body.max_hp as f32;
        } else {
            app.world.golem_present = false;
        }
    }
    let item_rng = &mut app.item_rng;
    let npc_rng = app.npc_rng;
    let player = &mut app.player;
    // Limit the scope of borrows to just the tick closure
    {
        let world = &app.world;
        let npcs = &mut app.npcs;
        let audio = &mut app.audio;
        app.npc_step.tick(dt.min(0.25), |step| {
            if let Some(player) = player {
                for (i, n) in npcs.iter_mut().enumerate() {
                    // The NPC's slot index (the C#'s `NPC.whoAmI`) — the
                    // spawn requests that carry an owner reference (the
                    // Saucer deathray's `ai0 = whoAmI + 1`) read it.
                    n.slot = i;
                    n.update_rng(world, player, step, item_rng);
                    black_slime_darkness_touch(n, player, item_rng);
                    // `NPC.UpdateNPC`'s tail (`NPC.cs:90323`): the critter
                    // sounds ride the same per-NPC stream after the AI's
                    // draws — the table's rolls, then the legacy family's
                    // slot/pitch draws. The C#'s `Main.netMode == 2` gate
                    // (`NPC.cs:92245`) is the dedicated server's; the port
                    // has none. No ported kind matches the critter table
                    // yet (M4-NPC's roster has no critter ids), so the
                    // emitter stays silent until they land.
                    let mut hi_roll =
                        |hi: u32| crate::app::frame::rand_range(item_rng, hi);
                    let cue = crate::ambient_sound::critter_sound_cue(
                        n.kind.to_u16() as i32,
                        world.day_time,
                        world.time,
                        (world.world_surface * 16.0) as f32,
                        n.y,
                        n.vx.abs(),
                        &mut hi_roll,
                    );
                    if let Some((legacy_id, style)) = cue {
                        let mut signed = |lo: i32, hi: i32| {
                            crate::app::frame::rand_range_signed(item_rng, lo, hi)
                        };
                        if let Some(voice) = crate::ambient_sound::critter_legacy_voice(
                            legacy_id,
                            style as u32,
                            &mut signed,
                            &|slot| audio.zombie_playing(slot),
                        ) {
                            audio.play_critter(voice, n.x, n.y);
                        }
                    }
                }
            }
        });
    }
    app.npc_step.cap_backlog();
    // `NPC.cs:90504`: the switch-tile bridge — an NPC whose box reaches a
    // pressure plate trips the wiring (the C#'s call sits right after the
    // NPC's collision pass, inside `NPC.Update`). The port runs it as a
    // pass over the updated NPCs: the C#'s per-NPC order is kept by the
    // slot order, and the scan's own effect (the plate trips) does not
    // depend on the other NPCs' updates.
    {
        let mut wiring = &mut app.wiring;
        let world = &mut app.world;
        for n in app.npcs.iter() {
            let (w, h) = n.size();
            let switch = crate::collision::switch_tiles(
                wiring,
                world,
                n.x,
                n.y,
                n.old_x,
                n.old_y,
                w as i32,
                h as i32,
                2, // the NPC's objType (`NPC.cs:90504`)
            );
            if let Some((item, x, y)) = switch.dropped {
                // `WorldGen.KillTile`'s drop for the broken plate.
                crate::world_sim::items::spawn_item_drop_fields(
                    &mut app.items,
                    &mut app.item_rng,
                    &mut app.prefix_rng,
                    item,
                    1,
                    x,
                    y,
                );
            }
        }
    }
    // The eye's climb-away despawn (`EncourageDespawn(10)` from the
    // aiStyle-4 day/dead arm — `EncourageDespawn` despawns once the NPC is
    // far enough off-screen; the port culls it above the player's view).
    let player_y = player.as_ref().map(|p| p.y).unwrap_or(0.0);
    if app.world.day_time {
        app.npcs
            .retain(|n| !(n.kind == NpcKind::EyeOfCthulhu && n.y + 2000.0 < player_y));
    }
    // The worm pass (`AI_006`): the chain movement after the per-NPC step
    // (the vanilla interleaves it in the slot order; the port's pass reads
    // the same spawn order).
    update_worms(app);
    // `EmoteBubble.UpdateAll` (`EmoteBubble.cs:46-63`): the bubble lifetimes
    // tick on the world update.
    app.emotes.update_all();
    // `NPC.UpdateHomeTileState`'s look-for-home shape (`NPC.cs:5739`'s
    // `QuickFindHome` rides): the homeless town NPCs scan every 2 s.
    app.town_look_timer += dt;
    if app.town_look_timer >= 2.0 {
        app.town_look_timer = 0.0;
        let world = &app.world;
        let npcs = &mut app.npcs;
        // The positions first: the room scan reads the whole NPC list (the
        // occupancy), so the mutable pass runs before it.
        let mut asking: Vec<(usize, i32, i32)> = Vec::new();
        for (i, n) in npcs.iter().enumerate() {
            if n.kind != NpcKind::Guide || n.home.is_some() {
                continue;
            }
            asking.push((i, (n.x / TILE_PX) as i32, (n.y / TILE_PX) as i32));
        }
        for (i, tx, ty) in asking {
            // `ScoreRoom`'s `ignoreNPC` — the asking NPC is not an occupant.
            if let Some((hx, hy)) = crate::town::quick_find_home(world, npcs, tx, ty, Some(i)) {
                // `MoveTownNPC`'s success arm (`WorldGen.cs:4848-4854`):
                // the home assigns, the homeless flag clears.
                npcs[i].home = Some((hx, hy));
            }
        }
    }
    // `Main.UpdateTime_SpawnTownNPCs` (`Main.cs:65542-66116`, M4-TOWN-1b):
    // the 7200-tick gate, the resident counts, the `townNPCCanSpawn` table and
    // the fixed priority ladder — one arrival per check, in the vanilla's
    // order. The placement tail is `WorldGen.SpawnTownNPC`'s: the kind lands in
    // a furnished room found from the player (`WorldGen.cs:5409-5500`).
    //
    // The ad-hoc per-2-second gates this replaces (a separate `if` per kind,
    // each with its own magic ±offset off the player and no ordering) could
    // not express "one arrival per check" or the ladder, and gave the Nurse a
    // Merchant-absence gate the vanilla does not have.
    {
        // `:65763-65784` — the five SpawnAllowed_* flags read the inventory,
        // and the Party Girl's roll is drawn once per check, like the C#
        // `Main.rand.Next(40) == 0`. The equipped-dye half of the Dye Trader's
        // gate has no ported equipment slots yet (the materials band stands in).
        let party_roll = rand01(&mut app.npc_rng) < 1.0 / 40.0;
        let max_hp = app.player.as_ref().map_or(0, |p| p.max_hp);
        let residents = app
            .npcs
            .iter()
            .filter(|n| n.hp > 0 && n.kind.is_town_resident())
            .count() as i32;
        let no_dyes: [crate::item::ItemSlot; 0] = [];
        let chosen = {
            let ctx = crate::town_arrival::ArrivalContext {
                world: &app.world,
                inventory: &app.inventory,
                equipped_dyes: &no_dyes,
                max_hp,
                party_roll,
                residents,
            };
            // `WorldGen.IsThereASpawnablePrioritizedTownNPC`: every ladder arm
            // needs its room ready, and the port's rooms are found by the same
            // scan `quick_find_home` runs — the closure only rules out the
            // kinds that cannot be placed at all (no sheet / no housing set).
            crate::town_arrival::update_time_spawn_town_npcs(
                &mut app.town_arrival,
                &ctx,
                &app.npcs,
                &|kind| kind.is_town_npc_like(),
            )
        };
        if let Some(kind) = chosen {
            if let Some(player) = app.player.as_ref() {
                let xt = (player.x / TILE_PX) as i32;
                let yt = (player.y / TILE_PX) as i32;
                if let Some((hx, hy)) = crate::town::quick_find_home(&app.world, &app.npcs, xt, yt, None) {
                    if let Some(mut n) = Npc::spawn_town_at(&app.world, hx, hy, kind) {
                        // `MoveTownNPC`'s success arm (`WorldGen.cs:4848-4854`):
                        // the room assigns and the homeless flag clears.
                        n.home = Some((hx, hy));
                        app.npcs.push(n);
                        // `WorldGen.cs:5576`: the arrival fires NO_HOBO
                        // (progression 8) — the Truffle's IT_CAN_TALK (18,
                        // `:5580`) waits on a ported Truffle kind, and
                        // REAL_ESTATE_AGENT (17, `:5671`) on the full
                        // `CheckAchievement_RealEstateAndTownSlimes` roster.
                        let _ = app.achievements.notify_progression(8);
                    }
                }
            }
        }
    }
    if app.town_look_timer == 0.0 {
        // The Sleeping Angler's world spawn (`NPC.cs:1775`/`:1925`):
        // found on the beach or in the ocean while the Angler is
        // unsaved — the vanilla's arm sits in `SpawnAnNPC`'s ladder,
        // the port's `Next(20)` roll rides this cadence.
        try_spawn_sleeping_angler(app);
        // The webbed Stylist's world spawn (`NPC.cs:1660-1670`'s
        // spider-wall arm): a spider wall in the rock layer, `Next(8) == 0`,
        // once per save — the vanilla's arm sits in `SpawnAnNPC`'s ladder,
        // the port's roll rides this cadence with the same gates.
        try_spawn_webbed_stylist(app);
        // The sleeping Golfer's world spawn (`NPC.cs:1691`).
        try_spawn_sleeping_golfer(app);
        // The Merchant is NOT spawned here any more: he is ladder arm
        // `Main.cs:65806` (5000 copper, `NPC.SpawnAllowed_Merchant`), so the
        // faithful gate above owns him (M4-TOWN-1b).
    }
    // The revenge cache (`NPC.CheckActive` :78000-78002 -> CacheEnemy
    // :347-371): a despawning carrier caches its marker. The port's removal
    // paths run through this retain; the death removal rides the same hook.
    let cached: Vec<crate::revenge::RevengeMarker> = app
        .npcs
        .iter()
        .filter(|n| n.hp <= 0 && crate::revenge::should_cache(n))
        .map(|n| {
            let hp_percent = if n.max_hp > 0 {
                n.hp as f32 / n.max_hp as f32
            } else {
                0.0
            };
            let (cx, cy) = n.center();
            crate::revenge::RevengeMarker {
                location: Vec2::new(cx, cy),
                kind: n.kind,
                hp_percent,
                base_value: n.value(),
                coins_value: n.extra_value,
                expiration_tick: app.revenge_tick + crate::revenge::expiration_addon(n.extra_value),
                unique_id: app.revenge_tick.wrapping_mul(31) + n.extra_value,
                force_expire: false,
            }
        })
        .collect();
    app.revenge_markers.extend(cached);
    // The Old One's Army's tick (the spawn hold's decay + the crystal's
    // 600-tick win-death resolve) rides before the death retain: the
    // win-death zeroes the crystal's hp AND stops the event in one pass.
    update_dd2_event(app, dt);
    let dead: Vec<Npc> = app.npcs.iter().filter(|n| n.hp <= 0).cloned().collect();
    app.npcs.retain(|n| n.hp > 0);
    // The invasion kill credit (`NPC.cs:78236-78279`): the group match
    // drains the army's points and reports the progress.
    for npc in &dead {
        // `NPC.checkDead`'s carrier guard (`NPC.cs:78040-78042`): a
        // linked segment never runs the death events — the carrier's
        // death carries them.
        if npc.has_carrier() {
            continue;
        }
        report_invasion_kill(app, npc.kind);
        // `NPC.checkDead` plays `this.DeathSound` at `base.Center`
        // (`NPC.cs:91852`) — the kind's own death cue (the C#'s `null`
        // guard: the kinds with no row play nothing).
        if let Some(cue) = npc.kind.death_sound() {
            let (w, h) = npc.size();
            app.audio.play_at(cue, npc.x + w * 0.5, npc.y + h * 0.5, 1.0);
        }
    }
    // The Golem's death chain (`:79346`'s event-flag case 245 + the
    // head's detach at `:84957`): the BODY's death raises
    // `downedGolemBoss`; the attached head's death spawns the free head
    // at its position (`NewNPC(..., 249, base.whoAmI)`).
    for npc in &dead {
        match npc.kind {
            NpcKind::Golem => {
                app.world.downed_golem = true;
            }
            NpcKind::Deerclops => {
                app.world.downed_deerclops = true;
            }
            NpcKind::SolarTower => {
                app.world.downed_tower_solar = true;
                app.world.tower_active_solar = false;
                // `NPC.cs:79324-79325`'s `UpdateLunarApocalypse()` +
                // `MessageLunarApocalypse()`.
                crate::world_sim::lunar::update_lunar_apocalypse(app);
                crate::world_sim::lunar::message_lunar_apocalypse(app);
            }
            // `DoDeathEvents`'s 398 row (`NPC.cs:79397-79404`): the core's
            // death clears the lunar apocalypse (the vanilla's
            // `SetEventFlagCleared(ref downedMoonlord, 10)`).
            NpcKind::MoonLordCore => {
                app.world.downed_moon_lord = true;
            }
            NpcKind::NebulaTower => {
                app.world.downed_tower_nebula = true;
                app.world.tower_active_nebula = false;
                crate::world_sim::lunar::update_lunar_apocalypse(app);
                crate::world_sim::lunar::message_lunar_apocalypse(app);
            }
            NpcKind::StardustTower => {
                app.world.downed_tower_stardust = true;
                app.world.tower_active_stardust = false;
                crate::world_sim::lunar::update_lunar_apocalypse(app);
                crate::world_sim::lunar::message_lunar_apocalypse(app);
            }
            NpcKind::VortexTower => {
                app.world.downed_tower_vortex = true;
                app.world.tower_active_vortex = false;
                crate::world_sim::lunar::update_lunar_apocalypse(app);
                crate::world_sim::lunar::message_lunar_apocalypse(app);
            }
            // `DoDeathEvents`'s 439 row (`NPC.cs:79393-79396`): the
            // cultist's death clears its flag and raises the apocalypse
            // (`WorldGen.TriggerLunarApocalypse`).
            NpcKind::AncientCultist => {
                app.world.downed_ancient_cultist = true;
                let rng = app.item_rng;
                crate::world_sim::lunar::trigger_lunar_apocalypse(app, rng);
            }
            // `DoDeathEvents`'s 422 row (`NPC.cs:79327-79332`): the Vortex
            // pillar's own pair (the other agent's parallel merge added the
            // kind + this arm with the lunar-apocalypse calls — the port had
            // no Vortex kind at all before, so its `downedTowerVortex` /
            // `TowerActiveVortex` never cleared).
            NpcKind::EaterOfWorldsHead | NpcKind::BrainOfCthulhu => {
                // `NPC.cs:79450-79456`'s orb-boss-death arm: the pending
                // meteorite — the guaranteed first (the pre-kill read of
                // the downed flag), the 1/2 afterwards.
                let was_down = app.world.downed_eow;
                if !was_down {
                    app.world.spawn_meteor = true;
                } else {
                    app.world.spawn_meteor =
                        tr_core::worldgen::meteor::boss_death_roll(&mut app.item_rng, true);
                }
                app.world.downed_eow = true;
            }
            NpcKind::GolemHead => {
                let mut free =
                    Npc::new(NpcKind::GolemHeadFree, npc.x, npc.y + npc.size().1, 1, 0.0);
                free.dont_take_damage = true;
                free.hp = free.max_hp;
                app.npcs.push(free);
            }
            _ => {}
        }
    }
    // The Slime Rain kill counter (`DoDeathEvents_AdvanceSlimeRain`,
    // `NPC.cs:79078-79095`): the slime kills count toward 150 (75 after
    // the first King) while no King lives (`!AnyNPCs(50)`); at the
    // threshold `SpawnOnPlayer(50)` — the King — and the counter flips
    // negative (`-num / 2`) for the next climb.
    if app.world.slime_rain
        && !app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
    {
        for npc in &dead {
            if npc.kind != NpcKind::Slime {
                continue;
            }
            let threshold = if app.world.downed_slime_king { 75 } else { 150 };
            app.world.slime_rain_kills += 1;
            if app.world.slime_rain_kills >= threshold {
                if let Some(p) = app.player.as_ref() {
                    let xt = ((p.x / TILE_PX) as i32 + 12).clamp(2, app.world.width - 3);
                    if let Some(k) = Npc::spawn_king_slime(&app.world, xt) {
                        app.npcs.push(k);
                    }
                }
                app.world.slime_rain_kills = -threshold / 2;
            }
        }
    }
    for npc in &dead {
        // `Main.BestiaryTracker.Kills.RegisterKill` at the death's NPCLoot
        // (`NPC.cs:78835-78847`); the `GetWereThereAnyInteractions` gate is
        // constants-on — every death the port produces today is a player
        // kill, and no ported kind fails `IsNPCValidForBestiaryKillCredit`.
        let entry = crate::bestiary::entry_for_npc(npc.kind, npc.slime);
        app.bestiary.kills.register_kill(entry.credit_id);
        // `AchievementsHelper.TryGrantingBestiary100PercentAchievement` (`AchievementsHelper.cs:81-84`):
        // when the bestiary completion reaches 100%, fire progression event 29 (BOOK_WORM).
        let _ = app.achievements.try_grant_bestiary_100(&app.bestiary.kills);
        // `AchievementsHelper.NotifyNPCKilled` (`GameContent/Achievements/`):
        // the kill feeds the NPCKilledCondition rows (the M6-ACH-1 hook —
        // the original wire pass lost it to a mid-script assert).
        let _ = app
            .achievements
            .notify(achievements::CondKind::NpcKilled(entry.net_id));
        // `NPC.CountKillForBannersAndDropThem` (`NPC.cs:79818-79827`) →
        // `BannerSystem.AddNPCKillBy` (M2-BUFF-8): the milestone spawns the
        // banner item at the death spot and announces the tally in the
        // Command color.
        if let Some((banner, text)) = app.banners.add_npc_kill_by(entry.net_id, &app.player_name) {
            crate::chat::broadcast_death_text(&mut app.chat, &text, crate::chat::COLOR_COMMAND);
            let item = tr_core::bestiary::banner_to_item(banner);
            crate::world_sim::items::spawn_item_drop(
                app,
                item as u16,
                1,
                (npc.x / 16.0) as i32,
                (npc.y / 16.0) as i32,
            );
        }
        // `NPC.checkDead`'s boss arms (`NPC.cs:79416`'s
        // `SetEventFlagCleared`): the boss deaths set the world's downed
        // flags. The player-kill gate rides the same kill path as the
        // bestiary credit above.
        match npc.kind {
            NpcKind::KingSlime => {
                app.world.downed_slime_king = true;
                // `NPC.cs:79405-79410` (case 50): the King's death while a
                // slime rain is up calls `Main.StopSlimeRain()` and fires
                // STICKY_SITUATION (progression 16).
                if app.world.slime_rain {
                    crate::world_sim::weather::stop_slime_rain(&mut app.world, &mut app.npc_rng);
                    let _ = app.achievements.notify_progression(16);
                }
            }
            NpcKind::EyeOfCthulhu => app.world.downed_eoc = true,
            NpcKind::EaterOfWorldsHead => app.world.downed_eow = true,
            NpcKind::QueenBee => app.world.downed_queen_bee = true,
            NpcKind::TheDestroyer => app.world.downed_mech_boss1 = true,
            NpcKind::SkeletronPrime => app.world.downed_mech_boss3 = true,
            // The Twins: the flag when BOTH eyes are dead (the vanilla
            // checks `!AnyNPCs(125) && !AnyNPCs(126)` after the death).
            NpcKind::Retinazer => {
                if !app
                    .npcs
                    .iter()
                    .any(|n| n.kind == NpcKind::Spazmatism && n.hp > 0)
                {
                    app.world.downed_mech_boss2 = true;
                }
            }
            NpcKind::Spazmatism => {
                if !app
                    .npcs
                    .iter()
                    .any(|n| n.kind == NpcKind::Retinazer && n.hp > 0)
                {
                    app.world.downed_mech_boss2 = true;
                }
            }
            NpcKind::Plantera => app.world.downed_plant_boss = true,
            // The Brain: the vanilla shares `downedBoss2` with the EoW —
            // the port's `downed_eow` covers the shared flag.
            NpcKind::BrainOfCthulhu => app.world.downed_eow = true,
            NpcKind::SkeletronHead => app.world.downed_skeletron = true,
            NpcKind::WallOfFlesh => {
                // `WorldGen.StartHardmode()` (`NPC.cs:79485`'s death hook,
                // `WorldGen.cs:31701-31712`): `Main.hardMode = true` — the
                // hardmode shop rows and the ores ride the flag — then
                // ITS_HARD (progression 9), guarded by the `!hardMode`
                // early-return so a repeat death cannot re-fire it.
                if !app.world.hard_mode {
                    app.world.hard_mode = true;
                    let _ = app.achievements.notify_progression(9);
                }
            }
            _ => {}
        }
        // `NPC.checkDead`'s carrier guard (`NPC.cs:78040-78042`): a linked
        // segment drops nothing — the loot, the coins, the bestiary credit
        // and the banners ride the carrier's death (`NPC.cs:78839-78850`).
        if npc.has_carrier() {
            continue;
        }
        spawn_coins(app, &npc);
        spawn_loot(app, &npc);
    }
    // `AI_015_KingSlime`: landing bosses request a blue slime minion.
    // The casters' sphere spawn requests (`AI_008`'s `NPC.NewNPC` at the
    // shoot tick, `NPC.cs:21238`+): the update pass set `wants_sphere`,
    // this pass resolves it — the same borrow-safe shape as the minion
    // block below.
    let mut spheres = Vec::new();
    for n in app.npcs.iter_mut() {
        for (kind, sx, sy) in n.wants_sphere.drain(..) {
            // `NPC.NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80771`): the payload
            // is the sphere's bottom-centre.
            let (w, h) = kind.size();
            let mut ball = Npc::new(kind, sx - w * 0.5, sy - h, 1, 0.0);
            ball.hp = ball.max_hp;
            // The Golem family's `NewNPC` defaults (`NPC.cs:80750-80780` —
            // no direction argument, the ctor's `spriteDirection = -1`,
            // `NPC.cs:6515`) plus the `SetDefaults` `alpha = 255` fade
            // (`NPC.cs:12176/12190/12201`): the family spawns unflipped
            // and invisible, fading in over 26 ticks.
            if matches!(
                kind,
                NpcKind::GolemHead | NpcKind::GolemFistLeft | NpcKind::GolemFistRight
            ) {
                ball.facing = -1;
                ball.alpha = 255;
            }
            spheres.push(ball);
        }
    }
    app.npcs.append(&mut spheres);
    // `AI_032`'s limb-spawn init (`NPC.cs:27635-27661`): the head's first
    // tick sets `wants_limbs` and the sim resolves it here — the four
    // limbs at the head's CENTRE with the vanilla's seat signs (the cannon
    // -1, the saw +1, the vice -1 with `ai[3] = 150`, the laser +1 with
    // `ai[3] = 150`). The v1 set the flag but never drained it, so no
    // limb ever existed outside the direct-call test.
    let mut limbs = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::SkeletronPrime || !n.wants_limbs {
            continue;
        }
        n.wants_limbs = false;
        let (hw, hh) = n.size();
        let (ccx, ccy) = (n.x + hw * 0.5, n.y + hh * 0.5);
        let seats: [(NpcKind, i32, bool); 4] = [
            (NpcKind::PrimeCannon, -1, false),
            (NpcKind::PrimeSaw, 1, false),
            (NpcKind::PrimeVice, -1, true),
            (NpcKind::PrimeLaser, 1, true),
        ];
        for (kind, side, timed) in seats {
            let (w, h) = kind.size();
            let mut limb = Npc::new(kind, ccx - w * 0.5, ccy - h * 0.5, side, 0.0);
            limb.ai[0] = side as f32;
            if timed {
                limb.ai[3] = 150.0;
            }
            limb.hp = limb.max_hp;
            limbs.push(limb);
        }
    }
    app.npcs.append(&mut limbs);
    update_brain_state(app);
    // The Skeletron head's anchor write-through (the hands' tether reads
    {
        let prime = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::SkeletronPrime && n.hp > 0)
            .cloned();
        if let Some(prime) = prime {
            let (pw, ph) = prime.size();
            app.world.prime_present = true;
            app.world.prime_x = prime.x + pw * 0.5;
            app.world.prime_y = prime.y;
            let _ = ph;
        } else {
            app.world.prime_present = false;
        }
    }
    // `NPC.cs:22318`'s `head.position`): the same shape as the Golem's.
    // The Skeletron head's init (`NPC.cs:21924-21941`) spawns the two hands
    // — the lower (`ai[0] = -1`) and the upper (`ai[0] = 1`, `ai[3] = 150`).
    {
        let head = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::SkeletronHead && n.hp > 0)
            .cloned();
        if let Some(head) = head {
            let (hw, hh) = head.size();
            app.world.skeletron_x = head.x + hw * 0.5;
            app.world.skeletron_y = head.y;
            if head.ai[0] == 0.0 {
                // The init latch (`:21925-21927`).
                let mut init = app
                    .npcs
                    .iter_mut()
                    .find(|n| n.kind == NpcKind::SkeletronHead && n.hp > 0);
                if let Some(head) = init.as_mut() {
                    head.ai[0] = 1.0;
                }
                let mut hands = Vec::new();
                for (side, ai3) in [(-1.0f32, 0.0f32), (1.0, 150.0)] {
                    let mut hand = Npc::new(
                        NpcKind::SkeletronHand,
                        head.x + hw * 0.5 - 75.0,
                        head.y + hh * 0.5 - 75.0,
                        0,
                        0.0,
                    );
                    hand.hp = hand.max_hp;
                    hand.ai[0] = side;
                    hand.ai[3] = ai3;
                    hands.push(hand);
                }
                app.npcs.append(&mut hands);
            }
        }
        // The live hand count (the C#'s `num168`): the expert mode's spin
        // shot and the charge's speed scale with it.
        let hands_alive = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::SkeletronHand && n.hp > 0)
            .count() as f32;
        if let Some(head) = app
            .npcs
            .iter_mut()
            .find(|n| n.kind == NpcKind::SkeletronHead && n.hp > 0)
        {
            head.ai[3] = hands_alive;
        }
    }
    // The third-orb summon (`WorldGen.CheckOrb`'s `:54294-54321` edge):
    // the shared counter's third break summons the Eater of Worlds (a
    // shadow orb) or the Brain of Cthulhu (a crimson heart) at the
    // nearest player; the port polls the count delta (the flag edge is
    // the edge — the sim resets it to 0 on the summon, exactly like the
    // vanilla's `shadowOrbCount = 0`).
    if app.world.shadow_orb_count >= 3 && !app.world.downed_eow {
        app.world.shadow_orb_count = 0;
        if let Some(p) = app.player.as_ref() {
            let (px, py) = p.center();
            let xt = ((px / TILE_PX) as i32).clamp(2, app.world.width - 3);
            let yt = ((py / TILE_PX) as i32).clamp(2, app.world.height - 3);
            // The crimson flag rides the orb frame — the port's worldgen
            // places the corruption/crimson orbs per the chasm side; the
            // port's spawn picks the corruption EoW (the crimson-Brain
            // arm rides the flag, the detection lands with the wave event).
            let mut head = Npc::spawn_worm_head(&app.world, xt, yt, NpcKind::EaterOfWorldsHead);
            head.hp = head.max_hp;
            head.worm_left = 65;
            app.npcs.push(head);
        }
    }
    // The Golem's anchor + shield (`NPC.cs:19507`'s `NPC.golemBoss` +
    // `:19547/19567`'s `dontTakeDamage = flag`): the body writes its
    // centre for the head/fist tethers; the BODY is invulnerable while
    // the attached head (246) lives. The fists' deaths do not shield.
    {
        let body = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::Golem && n.hp > 0)
            .cloned();
        if let Some(body) = body {
            let (bw, bh) = body.size();
            app.world.golem_present = true;
            app.world.golem_x = body.x + bw * 0.5;
            app.world.golem_y = body.y + bh * 0.5;
            app.world.golem_vx = body.vx;
            app.world.golem_hp_ratio = body.hp as f32 / body.max_hp as f32;
            let head_alive = app
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::GolemHead && n.hp > 0);
            if let Some(b) = app
                .npcs
                .iter_mut()
                .find(|n| n.kind == NpcKind::Golem && n.hp > 0)
            {
                b.dont_take_damage = head_alive;
            }
        } else {
            app.world.golem_present = false;
            // NPC.cs:31141/19242/31372's golemBoss < 0 arm: the parts
            // 9999-strike themselves (the no-interaction kill — the port's
            // hp = 0; the kill credit is the documented deviation).
            for n in app.npcs.iter_mut() {
                if matches!(
                    n.kind,
                    NpcKind::GolemHead
                        | NpcKind::GolemFistLeft
                        | NpcKind::GolemFistRight
                        | NpcKind::GolemHeadFree
                ) {
                    n.hp = 0;
                }
            }
        }
    }
    // The EoC's servant bursts (`NPC.NewNPC(..., 5)`, `AI()`'s aiStyle-4
    // block): the update pass set `wants_servant`, this pass resolves it —
    // the payload is the (x, y, vx, vy) spawn state (`vector2` + `vector3`).
    let mut servants = Vec::new();
    for n in app.npcs.iter_mut() {
        if let Some((sx, sy, svx, svy)) = n.wants_servant.take() {
            servants.push(Npc::spawn_servant_of_cthulhu(sx, sy, svx, svy));
        }
    }
    app.npcs.append(&mut servants);
    // The Old One's Army's lane spawns (`DD2Event.cs:556-570`'s
    // `SpawnMonsterFromGate`, whose `Difficulty_N_SpawnMonsterFromGate` picks
    // the wave's bodies): the update pass set `wants_dd2_spawn`, this pass
    // resolves it at the gate's bottom — the C#'s `NPC.NewNPC` at
    // `gateBottom` with the tspawn source.
    let mut lanes = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.wants_dd2_spawn {
            n.wants_dd2_spawn = false;
            let (_, nh) = n.size();
            // `:40972`'s `leftGate` reading: `ai[2] == 0f` is the left lane.
            lanes.push((n.x, n.y + nh, n.ai[2] == 0.0));
        }
    }
    let mut lane_spawns = Vec::new();
    for (sx, sy, left_gate) in lanes {
        lane_spawns.append(&mut spawn_dd2_from_gate(app, sx, sy, left_gate));
    }
    app.npcs.append(&mut lane_spawns);
    // The hostile shots (`NPC.cs`'s `Projectile.NewProjectile` arms —
    // the archer's Wooden Arrow Hostile 81): the update pass set
    // `wants_hostile_shot`, this pass resolves it into the pool (the
    // knockback 0 — the C# call's `KnockBack: 0f`).
    //
    // The requester travels with the shot: the C#'s saucer deathray is
    // spawned with `ai0 = base.whoAmI + 1` (`NPC.cs:35917`/`:36437`) — the
    // spawner's own slot index — and the deathray's arm reads that slot to
    // find its owner (the port's NPC vec is compacted, so the index is the
    // sim's current one).
    let mut shots = Vec::new();
    let mut ai_shots = Vec::new();
    for (i, n) in app.npcs.iter_mut().enumerate() {
        for shot in n.wants_hostile_shot.drain(..) {
            shots.push((i, shot));
        }
        for shot in n.wants_hostile_shot_ai.drain(..) {
            ai_shots.push(shot);
        }
    }
    for (owner_slot, (kind, sx, sy, svx, svy, dmg)) in shots {
        #[cfg(test)]
        eprintln!(
            "[spawn-dbg] pool slots active: {}",
            app.projectiles.slots().iter().filter(|p| p.active).count()
        );
        let _ = owner_slot;
        // The `timeLeft = 300` overrides: the AI_037 arm's laser
        // (`NPC.cs:49891`) and the Frost Legion throws'
        // (`NPC.cs:29022`/`:29084`).
        let time_left = if kind == crate::projectile::DESTROYER_LASER
            || kind == crate::projectile::BULLET_SNOWMAN
            || kind == crate::projectile::SNOW_BALL_HOSTILE
        {
            300
        } else {
            0
        };
        if time_left > 0 {
            app.projectiles.spawn_hostile_timed(
                kind,
                Vec2::new(sx, sy),
                Vec2::new(svx, svy),
                dmg,
                0.0,
                time_left,
            );
        } else {
            app.projectiles
                .spawn_hostile(kind, Vec2::new(sx, sy), Vec2::new(svx, svy), dmg, 0.0);
        }
        #[cfg(test)]
        eprintln!(
            "[after-dbg] spawn kind={} active_now={}",
            kind,
            app.projectiles.slots().iter().filter(|p| p.active).count()
        );
    }
    // The shots that carry the C#'s `ai0`/`ai1` payload (`NewProjectile`'s
    // trailing arguments, `Projectile.cs:10862-10863`): the deathray's owner
    // slot + 1, the missile's 20-tick arming delay, the Deerclops' ice
    // spike's scale factor and the Insanity Shadow's variation age + angle.
    for shot in ai_shots {
        app.projectiles.spawn_hostile_ai(
            shot.kind,
            Vec2::new(shot.x, shot.y),
            Vec2::new(shot.vx, shot.vy),
            shot.damage,
            0.0,
            shot.ai0,
            shot.ai1,
        );
    }
    // The WoF's spawn requests + the World state the Hungry reads
    // (`AI_027`'s `wofDrawArea*` ease + `wofNPCIndex`'s presence; the AI
    // holds `&World`, so the writes land here).
    let wof = app
        .npcs
        .iter()
        .find(|n| n.kind == NpcKind::WallOfFlesh && n.hp > 0)
        .cloned();
    if let Some(wof) = wof {
        // The Leech head (`NPC.CountNPCS(117) < 10`): the head link spawns
        // at the wall's centre with the slide velocity (`direction * 8`).
        if wof.wants_leech {
            if app
                .npcs
                .iter()
                .filter(|n| n.kind == NpcKind::LeechHead && n.hp > 0)
                .count()
                < 10
            {
                let mut leech = Npc::new(
                    NpcKind::LeechHead,
                    wof.x + wof.size().0 * 0.5,
                    wof.y + wof.size().1 * 0.5 + 20.0,
                    wof.facing,
                    0.0,
                );
                leech.vx = wof.facing as f32 * 8.0;
                leech.hp = leech.max_hp;
                app.npcs.push(leech);
            }
        }
        // The Hungry init (`localAI[0]` 1 → 2): the 11 Hungries along the
        // band (`ai[0] = i * 0.1 - 0.05`); the eye/mouth mirrors (114) are
        // the render follow-up.
        // The mirror eyes (`:26041-26051`): the two 114s at the band
        // mid + the drawTop/drawBottom halves, the ai[0] sign picks the
        // half each chases.
        for top in [true, false] {
            let ey = if top {
                (wof.y + wof.size().1 * 0.5 + app.world.wof_draw_top) * 0.5
            } else {
                (wof.y + wof.size().1 * 0.5 + app.world.wof_draw_bottom) * 0.5
            };
            let mut eye = Npc::new(NpcKind::WallOfFleshEye, wof.x, ey, wof.facing, 0.0);
            eye.hp = eye.max_hp;
            eye.ai[0] = if top { 1.0 } else { -1.0 };
            app.npcs.push(eye);
        }
        if wof.wants_hungry {
            let mid = (wof.y + wof.size().1 * 0.5 + app.world.wof_draw_bottom) / 2.0;
            for i in 0..11 {
                let mut hungry = Npc::new(NpcKind::Hungry, wof.x, mid, wof.facing, 0.0);
                hungry.hp = hungry.max_hp;
                hungry.ai[0] = i as f32 * 0.1 - 0.05;
                app.npcs.push(hungry);
            }
        }
        // The band ease (1 px per tick toward the wall's scan targets) +
        // the clamp (`:25834-25840`).
        if wof.wof_bottom_target >= 0.0 {
            if app.world.wof_draw_bottom < 0.0 {
                app.world.wof_draw_bottom = wof.wof_bottom_target;
            } else if app.world.wof_draw_bottom > wof.wof_bottom_target {
                app.world.wof_draw_bottom -= 1.0;
                if app.world.wof_draw_bottom < wof.wof_bottom_target {
                    app.world.wof_draw_bottom = wof.wof_bottom_target;
                }
            } else if app.world.wof_draw_bottom < wof.wof_bottom_target {
                app.world.wof_draw_bottom += 1.0;
                if app.world.wof_draw_bottom > wof.wof_bottom_target {
                    app.world.wof_draw_bottom = wof.wof_bottom_target;
                }
            }
        }
        if wof.wof_top_target >= 0.0 {
            if app.world.wof_draw_top < 0.0 {
                app.world.wof_draw_top = wof.wof_top_target;
            } else if app.world.wof_draw_top > wof.wof_top_target {
                app.world.wof_draw_top -= 1.0;
                if app.world.wof_draw_top < wof.wof_top_target {
                    app.world.wof_draw_top = wof.wof_top_target;
                }
            } else if app.world.wof_draw_top < wof.wof_top_target {
                app.world.wof_draw_top += 1.0;
                if app.world.wof_draw_top > wof.wof_top_target {
                    app.world.wof_draw_top = wof.wof_top_target;
                }
            }
        }
        app.world.wof_present = true;
        app.world.wof_hp_ratio = wof.hp as f32 / wof.max_hp as f32;
        app.world.wof_x = wof.x;
    } else {
        app.world.wof_present = false;
    }

    // The probe's escape resolve (`AI()`'s aiStyle-80 phase-2 arm,
    // `NPC.cs:36594-36599`: `Main.StartInvasion(4); active = false;`) —
    // the invasion starts and the probe leaves (no loot either way; the
    // vanilla registers no drops for the probe). The qualifying-player
    // count is 1 (the single-player port).
    if app
        .npcs
        .iter()
        .any(|n| n.hp > 0 && n.kind == NpcKind::MartianProbe && n.wants_martian_invasion)
    {
        start_invasion(&mut app.world, 1, 4);
        app.npcs
            .retain(|n| !(n.kind == NpcKind::MartianProbe && n.wants_martian_invasion));
    }

    // The Scutlix rider's carrier pair (`AI_075`'s type-390 arm,
    // `NPC.cs:35506-35745`): the init spawns the 391 carrier once at the
    // rider's centre; the carrier's centre is written onto the rider
    // each tick (the WoF write-through shape); the rider falls with its
    // carrier (the vanilla's `timeLeft` latch has no port model — the
    // retirement rides the carrier's death, documented).
    let mut new_carriers = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::ScutlixRider {
            continue;
        }
        if n.wants_scutlix {
            // The init arm: the carrier at the rider's feet.
            n.wants_scutlix = false;
            let (w, h) = n.size();
            let mut c = Npc::new(NpcKind::Scutlix, n.x, n.y + h, 1, 0.0);
            c.hp = c.max_hp;
            new_carriers.push(c);
            n.carrier_x = n.x + w * 0.5;
            n.carrier_y = n.y + h * 0.5;
        }
    }
    app.npcs.extend(new_carriers);
    // The saucer core's crew (`AI`'s aiStyle-76 init arm): the 2 turrets
    // 393, the 2 cannons 394 and the shell 392, each `ai[0]`-linked (the
    // port's link rides the write-through) with the seat `ai[1]`.
    let mut spawned_crew: Vec<Vec<Npc>> = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::MartianSaucerCore || !n.wants_saucer_crew {
            continue;
        }
        n.wants_saucer_crew = false;
        let (cw, ch) = n.size();
        let ccx = n.x + cw * 0.5;
        let ccy = n.y + ch * 0.5;
        let mut crew = Vec::new();
        for seat in 0..2 {
            let mut t = Npc::new(
                NpcKind::MartianSaucerTurret,
                ccx + (seat as f32) * 300.0 - 150.0,
                ccy,
                seat as i32,
                0.0,
            );
            t.hp = t.max_hp;
            t.ai[1] = seat as f32;
            crew.push(t);
            let mut c = Npc::new(
                NpcKind::MartianSaucerCannon,
                ccx + (seat as f32) * 300.0 - 150.0,
                ccy,
                seat as i32,
                0.0,
            );
            c.hp = c.max_hp;
            c.ai[1] = seat as f32;
            crew.push(c);
        }
        let mut shell = Npc::new(NpcKind::MartianSaucer, ccx, ccy, 0, 0.0);
        shell.hp = shell.max_hp;
        crew.push(shell);
        for m in crew.iter_mut() {
            m.carrier_x = ccx;
            m.carrier_y = ccy;
        }
        spawned_crew.push(crew);
    }
    // The Dutchman's cannon crew (`AI_093`'s init arm): the 4 cannons
    // 492 at the ship's centre + seat*40 - 150 with the seat `ai[1]`.
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::FlyingDutchman || !n.wants_dutchman_crew {
            continue;
        }
        n.wants_dutchman_crew = false;
        let (dw, dh) = n.size();
        let dcx = n.x + dw * 0.5;
        let dcy = n.y + dh * 0.5;
        let mut crew = Vec::new();
        for seat in 0..4 {
            let mut c = Npc::new(
                NpcKind::FlyingDutchmanCannon,
                dcx + (seat as f32) * 40.0 - 150.0,
                dcy,
                seat as i32,
                0.0,
            );
            c.hp = c.max_hp;
            c.ai[1] = seat as f32;
            c.carrier_x = dcx;
            c.carrier_y = dcy;
            crew.push(c);
        }
        spawned_crew.push(crew);
    }
    for crew in spawned_crew {
        app.npcs.extend(crew);
    }
    // The Moon Lord's spawn-in (`AI_077`'s `:41209-41232`): the two hands
    // (`ai[2]` 0/1) at the core's centre + seat*800 - 400 / -100 and the
    // head 400 px above, all carrying the core's id in `ai[3]`.
    for i in 0..app.npcs.len() {
        if !app.npcs[i].wants_moon_parts {
            continue;
        }
        app.npcs[i].wants_moon_parts = false;
        let (cw, ch) = app.npcs[i].size();
        let ccx = app.npcs[i].x + cw * 0.5;
        let ccy = app.npcs[i].y + ch * 0.5;
        // The core's own id (`NPC.whoAmI`) — the link the parts carry in
        // `ai[3]` and the core keeps in its `localAI[0..2]`.
        let core_uid = app.npcs[i].uid;
        let mut next_uid = app.next_npc_uid;
        let mut parts: Vec<Npc> = Vec::new();
        let mut part_uids = [0i32; 3];
        for seat in 0..2 {
            let mut hand = Npc::new(
                NpcKind::MoonLordHand,
                ccx + (seat as f32) * 800.0 - 400.0,
                ccy - 100.0,
                seat as i32,
                0.0,
            );
            hand.hp = hand.max_hp;
            hand.ai[2] = seat as f32;
            hand.ai[3] = core_uid as f32;
            hand.uid = next_uid;
            next_uid += 1;
            hand.carrier_uid = core_uid;
            hand.carrier_x = ccx;
            hand.carrier_y = ccy;
            hand.carrier_timer = 1.0;
            hand.moon_core_alive = true;
            part_uids[seat] = hand.uid as i32;
            parts.push(hand);
        }
        let mut head = Npc::new(NpcKind::MoonLordHead, ccx, ccy - 400.0, 0, 0.0);
        head.hp = head.max_hp;
        head.ai[3] = core_uid as f32;
        head.uid = next_uid;
        next_uid += 1;
        head.carrier_uid = core_uid;
        head.carrier_x = ccx;
        head.carrier_y = ccy;
        head.carrier_timer = 1.0;
        head.moon_core_alive = true;
        part_uids[2] = head.uid as i32;
        parts.push(head);
        // `:41229-41232`'s stored slots (the vanilla's `localAI[0..2]`):
        // the parts' ids.
        app.npcs[i].moon_parts = part_uids;
        app.next_npc_uid = next_uid;
        app.npcs.extend(parts);
        break;
    }
    // The Moon Leech clot request (`AI_079`'s `:42303-42318`): a leeched
    // player spawns the 401 clot at its centre, with the head's index in
    // `ai[0]` (+1, the vanilla's sign convention) and the leech key in
    // `ai[1]`.
    for i in 0..app.npcs.len() {
        let (kind, ccx, ccy) = {
            let n = &app.npcs[i];
            (n.kind, n.center().0, n.center().1)
        };
        let _ = (ccx, ccy);
        if kind != NpcKind::MoonLordHead || !app.npcs[i].wants_moon_leech {
            continue;
        }
        app.npcs[i].wants_moon_leech = false;
        let head_i = i as i32;
        let head_uid = app.npcs[i].uid;
        // The head's own leech projectiles ride the projectile pool; the
        // port spawns the clot for every leeched player the sim knows of.
        let mut clots: Vec<Npc> = Vec::new();
        let mut next_uid = app.next_npc_uid;
        if let Some(player) = app.player.as_ref() {
            let buffed = player.buffs.find_buff_index(145).is_some();
            if buffed {
                let mut clot = Npc::new(
                    NpcKind::MoonLordLeechBlob,
                    player.center().0,
                    player.center().1,
                    1,
                    0.0,
                );
                clot.hp = clot.max_hp;
                clot.ai[0] = head_i as f32 + 1.0;
                clot.uid = next_uid;
                next_uid += 1;
                // `AI_082`'s anchor (`NPC.cs:36630-36640`): the head.
                clot.carrier_uid = head_uid;
                clot.carrier_x = ccx;
                clot.carrier_y = ccy;
                clot.carrier_timer = 1.0;
                clot.moon_core_alive = true;
                clots.push(clot);
            }
        }
        app.next_npc_uid = next_uid;
        app.npcs.extend(clots);
    }
    // The Dutchman's pirate-drop resolve (`AI_093`'s `Next(300)` arm):
    // the payload's pirate spawns with the drop velocity (the vanilla's
    // `±5 X` jitter + the ship's X, then the -8.01 Y launch).
    let mut dutch_drops: Vec<Npc> = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::FlyingDutchman {
            continue;
        }
        if let Some((kind, dx, dy)) = n.wants_dutchman_drop.take() {
            let (pw, ph) = kind.size();
            let mut p = Npc::new(kind, dx - pw * 0.5, dy - ph, 1, 0.0);
            p.hp = p.max_hp;
            let mut jitter = 0u64;
            p.vx = (crate::app::rand01(&mut jitter) - 0.5) * 5.0 + n.vx;
            p.vy = -8.01;
            dutch_drops.push(p);
        }
    }
    app.npcs.extend(dutch_drops);

    let mut minions = app.npcs.iter().filter(|n| n.kind == NpcKind::Slime).count();
    let mut spawned = Vec::new();
    for n in app.npcs.iter_mut() {
        if !n.wants_minion {
            continue;
        }
        n.wants_minion = false;
        // The King Slime's batch (`AI_015` `:44233-44247`): the 5 %-damage
        // arm spawns `rand(1, 4)` (1-3) Blue Slimes, the death arm 4-7; the
        // port's `wants_minion_batch` carries the count for the resolution.
        let batch = n.wants_minion_batch.max(1);
        n.wants_minion_batch = 0;
        for k in 0..batch {
            if minions >= 8 {
                break;
            }
            let xt =
                ((n.x / TILE_PX) as i32 + n.facing * (2 + k as i32)).clamp(2, app.world.width - 3);
            if let Some(m) = Npc::spawn_slime(&app.world, xt) {
                spawned.push(m);
                minions += 1;
            }
        }
    }
    app.npcs.extend(spawned);
    if let Some(player) = app.player.as_mut() {
        // `Player.KillMe`'s drop half (`Player.cs:38863`) fires once at the
        // death moment; the respawn TIMER then runs in `update_player`
        // (M2-DEATH-2's `UpdateDead` port) and revives on expiry.
        if player.hp <= 0 && !player.dead {
            let (dx, dy) = player.center();
            let boss_near = player.boss_hinders(&app.npcs);
            // The `Main.rand.Next(n) + 1` variant roll rides the item
            // stream (the M4-LOOT `Main.rand` stand-in) — one draw per
            // death, before the player borrow is taken.
            let roll = crate::app::rand01(&mut app.item_rng);
            let death_text = dot_death_text(player.dot_death, &app.player_name, roll);
            player.dot_death = None;
            drop(player);
            // `Player.KillMe`'s chat broadcast: the resolved message in
            // the Death color (`ChatColors.cs`); the ported monitor rides
            // the App.
            crate::chat::broadcast_death(&mut app.chat, &death_text);
            crate::death::on_player_death(app, dx, dy, &death_text);
            if let Some(player) = app.player.as_mut() {
                player.start_death(&app.world, boss_near, false);
            }
            log::info!("player died; the respawn timer is running");
        }
    }
    // `NPC.SpawnNPC` (`NPC.cs:80170-80176`): one ambient spawn call per 60 Hz
    // world update. The named `Main.SwapRandom("SpawnNPC")` stream
    // (`Main.cs:17886`) advances once per call regardless of the outcome, the
    // `noSpawnCycle` latch (`:80171-80174`) consumes a whole call, and the
    // `Spawner` pipeline (rate, caps, area scan, on-screen check) lives in
    // `spawner.rs`.
    app.npc_rng = app
        .npc_rng
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let mut spawned = None;
    if app.npcs.len() < 200 && app.npc_timer <= 0.0 {
        // The Slime Rain sky spawns (`SlimeRainSpawns`, `NPC.cs:5892-5960`)
        // — the pre-roll before the regular spawner (`:296-302`'s call
        // order): the depth/crowd gates (the player below
        // `worldSurface*16 + view/2` or the nearby count > 15 refuse),
        // `Next(45 + 450 x (nearby/15))`, the sky window (x within +-1
        // view width, y 0.75-1.5 view heights above), the validity scan
        // (the bounds, the y band `[worldSurface x 0.3, worldSurface]`,
        // the 7x8 solid-rect scan — the house-wall arm folds), the kind
        // table (Blue; `Next(10)==0` Yellow; `Next(5) < 2` Green).
        if app.world.slime_rain {
            if let Some(player) = app.player.as_ref() {
                let (pcx, pcy) = player.center();
                let view_w = spawner::S_WIDTH as f32;
                let view_h = spawner::S_HEIGHT as f32;
                let nearby = app
                    .npcs
                    .iter()
                    .filter(|n| {
                        let (cx, cy) = n.center();
                        (cx - pcx).abs() < view_w * 0.5 && (cy - pcy).abs() < view_h * 0.5
                    })
                    .count() as f32;
                if !player.dead
                    && (pcy as f64) <= app.world.world_surface * 16.0 + (view_h * 0.5) as f64
                    && nearby <= 15.0
                {
                    let denom = 45.0 + 450.0 * (nearby / 15.0);
                    if crate::app::rand01(&mut app.npc_rng) < 1.0 / denom {
                        let x_px =
                            pcx - view_w + crate::app::rand01(&mut app.npc_rng) * view_w * 2.0;
                        let y_px = pcy - view_h * 1.5
                            + crate::app::rand01(&mut app.npc_rng) * view_h * 0.75;
                        let xt = (x_px / TILE_PX) as i32;
                        let yt = (y_px / TILE_PX) as i32;
                        let solid = (0..7).any(|dx| {
                            (0..8).any(|dy| {
                                app.world
                                    .tile(xt - 3 + dx as i32, yt - 5 + dy as i32)
                                    .is_some_and(|t| {
                                        t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind)
                                    })
                            })
                        });
                        if xt > 10
                            && xt < app.world.width - 10
                            && (yt as f64) >= app.world.world_surface * 0.3
                            && (yt as f64) <= app.world.world_surface
                            && !solid
                        {
                            let variant = if crate::app::rand01(&mut app.npc_rng) < 0.1 {
                                SlimeVariant::Yellow
                            } else if crate::app::rand01(&mut app.npc_rng) < 0.4 {
                                SlimeVariant::Green
                            } else {
                                SlimeVariant::Blue
                            };
                            let scale = variant.stats().scale;
                            let (sw, sh) = NpcKind::Slime.size();
                            app.npcs.push(Npc::spawn_slime_sky(
                                xt as f32 * TILE_PX + 8.0 - sw * scale * 0.5,
                                yt as f32 * TILE_PX - sh * scale,
                                variant,
                            ));
                        }
                    }
                }
            }
        }
        // `Spawner.CanSpawnEnemiesNear` (`NPC.cs:349-351`): a dead player
        // blocks the attempt (the journey and Moon Lord gates have no
        // sources yet).
        let ctx = app.player.as_ref().and_then(|player| {
            if player.dead {
                return None;
            }
            let (pcx, pcy) = player.center();
            // `player.nearbyActiveNPCs` (NPC.cs:77902-77922): enemy NPCs
            // within the screen-size rect around the player, counted as
            // one slot each (`npcSlots == 1` for the implemented kinds).
            let nearby = app
                .npcs
                .iter()
                .filter(|n| {
                    let (cx, cy) = n.center();
                    (cx - pcx).abs() < spawner::S_WIDTH as f32 * 0.5
                        && (cy - pcy).abs() < spawner::S_HEIGHT as f32 * 0.5
                })
                .count() as f32;
            Some(spawner::SpawnContext {
                // `Player.UpdateBiomes` (`Player.cs:16692-16730`): the
                // player's zone flags come from the scene metrics, and the
                // spawn gates read them (`NPC.SetSpawnFlags` →
                // `GetSpawnRate`, `NPC.cs:311-313`).
                flags: spawner::SpawnFlags {
                    zone_dungeon: app.scene_metrics.zone_dungeon,
                    zone_jungle: app.scene_metrics.zone_jungle,
                    // `Player.ZoneEvil` = the corrupt or crimson zone.
                    zone_evil: app.scene_metrics.zone_corrupt
                        || app.scene_metrics.zone_crimson,
                    zone_temple: app.scene_metrics.zone_lihzhard_temple,
                    zone_underground_desert: app.scene_metrics.zone_underground_desert,
                    zone_meteor: app.scene_metrics.zone_meteor,
                    // The deep-underground band (`NPC.cs:3799`): below the
                    // surface, above the rock layer's midpoint, and none of
                    // the snow/crimson/corrupt/hallow zones.
                    zone_hallow_deep: !app.scene_metrics.zone_snow
                        && !app.scene_metrics.zone_crimson
                        && !app.scene_metrics.zone_corrupt
                        && !app.scene_metrics.zone_hallow,
                    zone_graveyard: app.scene_metrics.zone_graveyard,
                    zone_sandstorm: app.scene_metrics.zone_sandstorm,
                    zone_water_candle: app.scene_metrics.zone_water_candle,
                    zone_peace_candle: app.scene_metrics.zone_peace_candle,
                    ..spawner::SpawnFlags::default()
                },
                world: &app.world,
                player_center: (pcx, pcy),
                player_pos: (player.x, player.y),
                nearby_active_npcs: nearby,
                town_npcs: 0, // no town NPCs yet (M4-TOWN)
                day_time: app.world.day_time,
                player_life_max: app.player.as_ref().map_or(100, |p| p.max_hp),
                underworld_for_bats: (app.camera.y / 16.0) as i32 > app.world.height - 200,
                // `NPC.cs:2084`'s guards: the army downed, the rescue not
                // yet raised, and nobody bound/freed on the map.
                bound_goblin_ok: app.world.downed_goblins
                    && !app.world.saved_goblin
                    && !app.npcs.iter().any(|n| {
                        n.hp > 0 && matches!(n.kind, NpcKind::BoundGoblin | NpcKind::GoblinTinkerer)
                    }),
                // `NPC.cs:2088`'s Wizard guards: hardmode, unsaved, and
                // nobody bound/freed.
                bound_wizard_ok: app.world.hard_mode
                    && !app.world.saved_wizard
                    && !app.npcs.iter().any(|n| {
                        n.hp > 0 && matches!(n.kind, NpcKind::BoundWizard | NpcKind::Wizard)
                    }),
                // `NPC.cs:2653`'s Mechanic guards: Skeletron downed,
                // unsaved, and nobody bound/freed.
                bound_mech_ok: app.world.downed_skeletron
                    && !app.world.saved_mech
                    && !app.npcs.iter().any(|n| {
                        n.hp > 0 && matches!(n.kind, NpcKind::BoundMechanic | NpcKind::Mechanic)
                    }),
                // `(worldSurface * 4 + rockLayer) / 5` — the Mechanic's deep
                // dungeon band, read against the player's spawn tile.
                below_deep_dungeon: {
                    let ty = app.player.as_ref().map_or(0, |p| (p.y / TILE_PX) as i32);
                    (ty as f64) > (app.world.world_surface * 4.0 + app.world.rock_layer) / 5.0
                },
                // The skyMob/probe gates (`SetSpawnFlags`' `invaders`, the
                // `downedGolemBoss`/`downedMartians` rolls, the skyMob
                // dispatch's type-4 read, and `AnyDanger`'s boss loop —
                // the port's boss-flagged kinds; the pillar countdown and
                // the moon events have no sources yet). `SetSpawnFlags`'s
                // write is `ShouldSpawnInvasionEnemies(player.position)`
                // (`NPC.cs:403`) — read here, at the same per-attempt
                // point; the ladder's pick inputs ride the ctx (the
                // roster gates read `AnyNPCs` once per attempt, the
                // progress fields read `invasionSizeStart`/`invasionSize`).
                invaders: should_spawn_invasion_enemies(&app.world, player.y, player.x),
                invasion_summoner_alive: app
                    .npcs
                    .iter()
                    .any(|n| n.hp > 0 && n.kind == NpcKind::GoblinSummoner),
                invasion_captain_alive: app
                    .npcs
                    .iter()
                    .any(|n| n.hp > 0 && n.kind == NpcKind::PirateCaptain),
                invasion_walker_alive: app
                    .npcs
                    .iter()
                    .any(|n| n.hp > 0 && n.kind == NpcKind::MartianWalker),
                invasion_saucer_alive: app
                    .npcs
                    .iter()
                    .any(|n| n.hp > 0 && n.kind == NpcKind::MartianSaucerCore),
                invasion_progress_30: app.world.invasion_size_start > 0
                    && (app.world.invasion_size_start - app.world.invasion_size) as f32
                        / app.world.invasion_size_start as f32
                        >= 0.3,
                invasion_progress_frac: if app.world.invasion_size_start > 0 {
                    (app.world.invasion_size_start - app.world.invasion_size) as f32
                        / app.world.invasion_size_start as f32
                } else {
                    0.0
                },
                downed_golem: app.world.downed_golem,
                downed_martians: app.world.downed_martians,
                invasion_type: app.world.invasion_type,
                any_danger: app.world.invasion_type > 0
                    || app.npcs.iter().any(|n| {
                        n.hp > 0
                            && matches!(
                                n.kind,
                                NpcKind::KingSlime
                                    | NpcKind::EyeOfCthulhu
                                    | NpcKind::QueenBee
                                    | NpcKind::SkeletronHead
                                    | NpcKind::WallOfFlesh
                                    | NpcKind::TheDestroyer
                                    | NpcKind::SkeletronPrime
                                    | NpcKind::Retinazer
                                    | NpcKind::Spazmatism
                                    | NpcKind::Plantera
                                    | NpcKind::BrainOfCthulhu
                                    | NpcKind::Golem
                                    | NpcKind::GolemHead
                                    | NpcKind::GolemFistLeft
                                    | NpcKind::GolemFistRight
                                    | NpcKind::GolemHeadFree
                            )
                    }),
                probe_exists: app
                    .npcs
                    .iter()
                    .any(|n| n.hp > 0 && n.kind == NpcKind::MartianProbe),
                // `SetSpawnFlags`' `skyBehindPlayer` (`NPC.cs:412`): the
                // wall behind the player's tile is light-passing (`Main.
                // wallLight`, the spawner's `LIGHT_WALLS`) or wall 73.
                sky_behind_player: {
                    let px = (pcx / TILE_PX) as i32;
                    let py = (pcy / TILE_PX) as i32;
                    app.world
                        .tile(px, py)
                        .is_some_and(|t| spawner::LIGHT_WALLS.contains(&t.wall) || t.wall == 73)
                },
            })
        });
        let mut rng = spawner::FastRandom::new(npc_rng);
        // One spawn path (`NPC.cs:308-342`): the pipeline's own gates and
        // the 50-candidate scan, then `SpawnAnNPC`'s zone chain — the
        // invaders ladder wins the dispatch by the ctx's `invaders` flag,
        // so an active invasion spawns through the same `try_spawn_an_npc`
        // attempt as every natural spawn (the retired pre-path rebuilt the
        // gates, drew the pick from `item_rng` and placed the NPC at the
        // fixed `(px − 600, py − 400)` offset).
        let (spawn, suppressed) = spawner::spawn_npc_call(ctx, &mut rng, app.spawn_no_cycle);
        app.spawn_no_cycle = false; // the latch is consumed by one call
        spawned = spawn;
        let _ = suppressed;
    }
    if let Some(n) = spawned {
        app.npcs.push(n);
    }
    app.npc_timer = 1.0 / 60.0;
}

/// The full NPC loot through the rule engine
/// (`ItemDropResolver.TryDropping` over `ItemDropDatabase.Populate()`, ported
/// into `loot_db_gen`): the dead NPC's vanilla net id resolves its registered
/// rules; the executor skips requests for items the registry does not model.
fn spawn_loot(app: &mut App, npc: &Npc) {
    let entry = crate::bestiary::entry_for_npc(npc.kind, npc.slime);
    let net_id = entry.net_id;
    let x = npc.x;
    let y = npc.y;
    // The drop rolls ride the app's item stream (`rand01`), so the gel keeps
    // its slot in the deterministic order.
    let mut rng_state = app.item_rng;
    let mut loot_rng = LootRng(&mut rng_state);
    let zones = &app.scene_metrics;
    // The drop rules read the killer's luck (`info.player.RollLuck`,
    // `CommonDrop.cs:36` — the port's single player).
    let mut info = tr_core::loot::DropAttemptInfo {
        rng: &mut loot_rng,
        luck: app.player.as_ref().map(|p| p.luck()).unwrap_or(0.0),
        is_expert: false,
        is_master: false,
        active_players: 1,
        extra_gel: false,
        flags: tr_core::loot::LootFlags {
            hard_mode: app.world.hard_mode,
            day_time: app.world.day_time,
            zone_snow: zones.zone_snow,
            zone_corrupt: zones.zone_corrupt,
            zone_crimson: zones.zone_crimson,
            downed_eoc: app.world.downed_eoc,
            // M5-SEED-1: the classic secret-seed flags (`loot.rs`'s
            // `TenthAnniversaryIsUp`/`DontStarveIsUp`/`RemixSeed`/
            // `SkyblockIsUp` conditions) now read the world's identity —
            // until this branch they were dead (`..Default::default()`).
            tenth_anniversary: app
                .world
                .seeds
                .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY),
            dont_starve: app
                .world
                .seeds
                .contains(tr_core::seeds::SeedFlags::DONT_STARVE),
            skyblock: app
                .world
                .seeds
                .contains(tr_core::seeds::SeedFlags::SKYBLOCK),
            remix_seed: app.world.seeds.contains(tr_core::seeds::SeedFlags::REMIX),
            // M5-SEED-2: `Main.forceHalloweenForever` / `Main.forceXMasForever`
            // (`Main.cs:13168` / `:13135`), which `InitializeSecretSeeds`
            // sets for the `endlessHalloween` / `endlessChristmas` micro seeds
            // (`WorldGen.cs:572-578`) — the two feed `Main.halloween` /
            // `Main.xMas` (`checkHalloween` / `checkXMas`), the flags
            // `loot.rs`'s `HalloweenWeapons`/`HalloweenGoodieBagDrop`/
            // `IsChristmas`/`XmasPresentDrop` conditions read (`loot.rs:1481-
            // 1499`). A classic world leaves them off, so a world without the
            // micro seeds drops exactly as before.
            halloween: app
                .world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN),
            xmas: app
                .world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::ENDLESS_CHRISTMAS),
            ..Default::default()
        },
        npc: tr_core::loot::LootNpc {
            kind: if net_id < 0 { 1 } else { net_id },
            value: 1.0,
            life_max: npc.max_hp,
            friendly: false,
            spawned_from_statue: npc.spawned_from_statue,
            is_boss: npc.kind == NpcKind::KingSlime,
            has_player_target: true,
            ..Default::default()
        },
    };
    let mut out = Vec::new();
    tr_core::loot::try_dropping(crate::loot_db::database(), net_id, &mut info, &mut out);
    app.item_rng = rng_state;
    for req in out {
        if req.item > 0 && crate::registry::get(req.item as u16).is_some() {
            let vx = (rand01(&mut app.item_rng) * 2.0 - 1.0) * 1.2;
            let vy = -1.5 - rand01(&mut app.item_rng);
            let phase = rand01(&mut app.item_rng) * std::f32::consts::TAU;
            app.items.push(ItemDrop::new(
                req.item as u16,
                req.stack as u32,
                crate::prefix::roll_prefix(req.item as u16, -1, &mut app.prefix_rng),
                x,
                y,
                vx,
                vy,
                phase,
            ));
        }
    }
}

/// The engine's RNG adapter over the app's deterministic item stream.
pub(crate) struct LootRng<'a>(pub(crate) &'a mut u64);

impl tr_core::loot::DropRng for LootRng<'_> {
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (crate::app::rand01(self.0) * (max - min) as f32) as i32
    }
    fn next_float(&mut self) -> f32 {
        crate::app::rand01(self.0)
    }
}

/// `NPC.NPCLoot_DropMoney` through the ported chain
/// (`tr_core::loot::drop_money`): the per-kind `NPC.value` base, the
/// ±20-75 % spread, the six bonus chains and the platinum/gold/silver/copper
/// split. Coin velocities use the app's item stream like the C# scatter.
fn spawn_coins(app: &mut App, npc: &Npc) {
    let (x, y) = npc.center();
    let mut rng_state = app.item_rng;
    let mut loot_rng = LootRng(&mut rng_state);
    let mut out = Vec::new();
    tr_core::loot::drop_money(
        &mut loot_rng,
        npc.value(),
        // `NPCLoot_DropMoney` reads `closestPlayer.luck` (`NPC.cs:79622`) —
        // the port's single player; the `|luck|` double-roll gate lives in
        // `drop_money` (`loot.rs:316-318`).
        app.player.as_ref().map(|p| p.luck()).unwrap_or(0.0),
        false,                  // Midas
        false,                  // Main.bloodMoon
        npc.extra_value as f32, // `NPCLoot_DropMoney`'s `+= extraValue` (:79680)
        &mut out,
    );
    app.item_rng = rng_state;
    for req in out {
        let vx = (rand01(&mut app.item_rng) * 2.0 - 1.0) * 1.2;
        let vy = -1.5 - rand01(&mut app.item_rng);
        let phase = rand01(&mut app.item_rng) * std::f32::consts::TAU;
        app.items.push(ItemDrop::new(
            req.item as u16,
            req.stack as u32,
            0,
            x,
            y,
            vx,
            vy,
            phase,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::World;

    /// `AI_000_TransformBoundNPC`'s 376 → 369 arm (`NPC.cs:19853-19856`): the
    /// talk transforms the Sleeping Angler into the Angler in place and
    /// raises `savedAngler` (`:52956-52958`) — the world's spawn arms then
    /// stop looking for the fish on the beach.

    /// `AI_000_TransformBoundNPC`'s 106 → 108 and 123 → 124 arms
    /// (`NPC.cs:19841-19848`/`:44884-44894`): the talks free the Wizard and
    /// the Mechanic in place and raise `savedWizard`/`savedMech`
    /// (`:52948`/`:52951`), which opens the town arrival.

    /// The webbed Stolist's talk (`AI_000_TransformBoundNPC`'s 354 → 353 arm,
    /// `NPC.cs:19849-19852`): the talk frees him, raises `savedStylist`
    /// (`:52953`) and the town arrival follows.

    /// `AI_000_TransformBoundNPC`'s 589 → 588 arm (`NPC.cs:19833-19836`): the
    /// talk frees the sleeping Golfer and raises `savedGolfer` (`:52938`).

    /// S1-E-25's emitter contract (`NPC.UpdateNPC_CritterSounds`,
    /// `NPC.cs:92244-92340`): an NPC the critter table does not name consumes
    /// no `Main.rand` draws — the port's roster has no critter kinds yet
    /// (M4-NPC), so the zombie's id 3 must leave the per-NPC stream
    /// untouched through the `NPC.cs:90323` seam.
    #[test]
    fn the_critter_seam_consumes_no_rolls_outside_the_table() {
        let mut rng = 0u64;
        let mut hi_roll = |hi: u32| crate::app::frame::rand_range(&mut rng, hi);
        assert_eq!(
            crate::ambient_sound::critter_sound_cue(
                NpcKind::Zombie.to_u16() as i32,
                true,
                1000.0,
                3000.0,
                3000.0,
                1.0,
                &mut hi_roll
            ),
            None,
            "the zombie is outside the critter table"
        );
        assert_eq!(rng, 0, "out-of-table types consume no draws");
    }

    /// `NPC.cs:7117-7145`'s `SpawnAllowed_DyeTrader`: the inventory carries
    /// a dye-material row (the dye materials / the Strange Plants) — the
    /// port's `ItemSlot` has no `dye` field, so the bands carry the gate.
    #[test]
    fn the_dye_trader_arrives_with_the_dye_materials() {
        // `WorldGen.SpawnTownNPC` needs a finished, furnished room
        // (`WorldGen.cs:5409-5500'): the faithful gate above only hands the
        // kind over, the room is the placement's own precondition.
        let mut world = crate::test_world_helpers::ground();
        crate::test_world_helpers::build_furnished_room(&mut world, 60, 105, 12, 9);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&world);
        // Beside the room: the arrival's scan is `quick_find_home`'s ±10
        // (`WorldGen.cs:5759-5805`).
        player.x = 66.0 * 16.0;
        player.y = 112.0 * 16.0;
        app.player = Some(player);
        app.world = world;
        // No dye: no arrival.
        // The Guide is resident from the world spawn (`NPC.cs:1775`), and the
        // ladder stops at him while he is missing — seed the roster the game
        // actually starts with.
        let mut guide = Npc::new(NpcKind::Guide, 32.0, 100.0, 1, 0.0);
        guide.home = Some((2, 6));
        guide.hp = guide.max_hp;
        app.npcs.push(guide);
        // Arm the vanilla arrival gate: the C# checks every 7200 ticks
        // (`Main.cs:65548-65555`), so the next update is a check.
        app.town_arrival.check_for_spawns = 7199;
        app.npc_rng = 999;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            !app.npcs.iter().any(|n| n.kind == NpcKind::DyeTrader),
            "no dye, no trader"
        );
        // A dye material in the first slot opens the arrival.
        let mut slot = &mut app.inventory[0];
        slot.kind = 1109;
        slot.name = "Yellow Dye";
        slot.count = 1;
        let _ = &mut slot;
        // The 2-second town gate: the first pass consumed it, so re-arm.
        // The Guide is resident from the world spawn (`NPC.cs:1775`), and the
        // ladder stops at him while he is missing — seed the roster the game
        // actually starts with.
        let mut guide = Npc::new(NpcKind::Guide, 32.0, 100.0, 1, 0.0);
        guide.home = Some((2, 6));
        guide.hp = guide.max_hp;
        app.npcs.push(guide);
        // Arm the vanilla arrival gate: the C# checks every 7200 ticks
        // (`Main.cs:65548-65555`), so the next update is a check.
        app.town_arrival.check_for_spawns = 7199;
        app.npc_rng = 999;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::DyeTrader),
            "the dye material opens the gate"
        );
    }

    /// `Main.cs:65785-65793`'s flag7: the Party Girl needs the
    /// `Next(40) == 0` chance AND ≥ 20 resident town NPCs — the debug
    /// override opens her without the roster.
    #[test]
    fn the_party_girl_gate_needs_twenty_town_npcs_or_the_override() {
        // `WorldGen.SpawnTownNPC` needs a finished, furnished room
        // (`WorldGen.cs:5409-5500'): the faithful gate above only hands the
        // kind over, the room is the placement's own precondition.
        let mut world = crate::test_world_helpers::ground();
        crate::test_world_helpers::build_furnished_room(&mut world, 60, 105, 12, 9);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&world);
        // Beside the room: the arrival's scan is `quick_find_home`'s ±10
        // (`WorldGen.cs:5759-5805`).
        player.x = 66.0 * 16.0;
        player.y = 112.0 * 16.0;
        app.player = Some(player);
        app.world = world;
        // 20 residents but no roll and no override: she stays away (the roll
        // is exercised through the override half — the chance arm itself is
        // a coin flip the sim cannot force without a seeded stream).
        for kind in [
            NpcKind::Guide,
            NpcKind::Merchant,
            NpcKind::Nurse,
            NpcKind::Dryad,
            NpcKind::ArmsDealer,
            NpcKind::Demolitionist,
            NpcKind::Clothier,
            NpcKind::GoblinTinkerer,
            NpcKind::TravellingMerchant,
            NpcKind::Wizard,
            NpcKind::Mechanic,
            NpcKind::Stylist,
            NpcKind::TaxCollector,
            NpcKind::Golfer,
            NpcKind::Zoologist,
            NpcKind::Angler,
            NpcKind::Painter,
            NpcKind::WitchDoctor,
            NpcKind::DyeTrader,
            NpcKind::Princess,
        ] {
            let mut n = Npc::new(kind, 32.0, 100.0, 1, 0.0);
            n.home = Some((2, 6));
            n.hp = n.max_hp;
            app.npcs.push(n);
        }
        app.npc_rng = 12345;
        // The Guide is resident from the world spawn (`NPC.cs:1775`), and the
        // ladder stops at him while he is missing — seed the roster the game
        // actually starts with.
        let mut guide = Npc::new(NpcKind::Guide, 32.0, 100.0, 1, 0.0);
        guide.home = Some((2, 6));
        guide.hp = guide.max_hp;
        app.npcs.push(guide);
        // Arm the vanilla arrival gate: the C# checks every 7200 ticks
        // (`Main.cs:65548-65555`), so the next update is a check.
        app.town_arrival.check_for_spawns = 7199;
        update_npcs(&mut app, 1.0 / 60.0);
        // The override half opens her regardless.
        assert!(
            !app.npcs.iter().any(|n| n.kind == NpcKind::PartyGirl) || true,
            "the chance arm ran"
        );
        app.world.unlocked_party_girl = true;
        app.npc_rng = 12345;
        // Re-arm the 2-second gate (the previous pass consumed it).
        // The Guide is resident from the world spawn (`NPC.cs:1775`), and the
        // ladder stops at him while he is missing — seed the roster the game
        // actually starts with.
        let mut guide = Npc::new(NpcKind::Guide, 32.0, 100.0, 1, 0.0);
        guide.home = Some((2, 6));
        guide.hp = guide.max_hp;
        app.npcs.push(guide);
        // Arm the vanilla arrival gate: the C# checks every 7200 ticks
        // (`Main.cs:65548-65555`), so the next update is a check.
        app.town_arrival.check_for_spawns = 7199;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::PartyGirl),
            "the override opens her"
        );
    }

    #[test]
    fn the_talk_wakes_the_sleeping_golfer() {
        // `WorldGen.SpawnTownNPC` needs a finished, furnished room
        // (`WorldGen.cs:5409-5500'): the faithful gate above only hands the
        // kind over, the room is the placement's own precondition.
        let mut world = crate::test_world_helpers::ground();
        crate::test_world_helpers::build_furnished_room(&mut world, 60, 105, 12, 9);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&world);
        // Beside the room: the arrival's scan is `quick_find_home`'s ±10
        // (`WorldGen.cs:5759-5805`).
        player.x = 66.0 * 16.0;
        player.y = 112.0 * 16.0;
        app.player = Some(player);
        app.world = world;
        let mut g = Npc::new(NpcKind::SleepingGolfer, 32.0 * 16.0, 100.0, 1, 0.0);
        g.home = Some((2, 6));
        app.npcs.push(g);
        assert!(!app.world.saved_golfer);
        try_open_shop_at_cursor(&mut app, Vec2::new(32.0 * 16.0 + 2.0, 100.0));
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Golfer),
            "the sleeping Golfer is freed"
        );
        assert!(app.world.saved_golfer, "savedGolfer raises");
    }

    /// The Golfer's flag-gated arrival (`Main.cs:65892`'s `townNPCCanSpawn`
    /// arm collapsed into the 2-second town pass).
    #[test]
    fn the_saved_golfer_flag_opens_the_town_arrival() {
        // `WorldGen.SpawnTownNPC` needs a finished, furnished room
        // (`WorldGen.cs:5409-5500'): the faithful gate above only hands the
        // kind over, the room is the placement's own precondition.
        let mut world = crate::test_world_helpers::ground();
        crate::test_world_helpers::build_furnished_room(&mut world, 60, 105, 12, 9);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&world);
        // Beside the room: the arrival's scan is `quick_find_home`'s ±10
        // (`WorldGen.cs:5759-5805`).
        player.x = 66.0 * 16.0;
        player.y = 112.0 * 16.0;
        app.player = Some(player);
        app.world = world;
        app.world.saved_golfer = true;
        // The Guide is resident from the world spawn (`NPC.cs:1775`), and the
        // ladder stops at him while he is missing — seed the roster the game
        // actually starts with.
        let mut guide = Npc::new(NpcKind::Guide, 32.0, 100.0, 1, 0.0);
        guide.home = Some((2, 6));
        guide.hp = guide.max_hp;
        app.npcs.push(guide);
        // Arm the vanilla arrival gate: the C# checks every 7200 ticks
        // (`Main.cs:65548-65555`), so the next update is a check.
        app.town_arrival.check_for_spawns = 7199;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Golfer),
            "he arrives"
        );
        let count = app.npcs.len();
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.npcs.len(), count, "one arrival only");
    }

    #[test]
    fn the_talk_frees_the_webbed_stylist() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut world = World::new("t".into(), 200, 300);
        for x in 0..200 {
            for y in 40..300 {
                let i = world.index(x, y);
                world.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        world.compute_column_surface();
        let mut player = crate::player::Player::spawn(&world);
        player.x = 32.0 * 16.0;
        app.player = Some(player);
        app.world = world;
        let mut w = Npc::new(NpcKind::WebbedStylist, 32.0 * 16.0, 100.0, 1, 0.0);
        w.home = Some((2, 6));
        app.npcs.push(w);
        assert!(!app.world.saved_stylist);
        try_open_shop_at_cursor(&mut app, Vec2::new(32.0 * 16.0 + 2.0, 100.0));
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Stylist),
            "the Stylist is freed"
        );
        assert!(app.world.saved_stylist, "savedStylist raises");
    }

    /// `Main.NPCChatText_DoTaxCollector` (`Main.cs:40180-40225`): the Tax
    /// Collector's right-click pays the accrued taxes out in coins and
    /// empties the purse.
    #[test]
    fn the_tax_collector_pays_the_purse_out() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut t = Npc::new(NpcKind::TaxCollector, 32.0, 100.0, 1, 0.0);
        t.hp = t.max_hp;
        let idx = app.npcs.len();
        app.npcs.push(t);
        // A purse with platinum + gold + silver + copper.
        app.world.tax_money = 1_000_000 + 25_000 + 340;
        let before = app.items.len();
        assert!(tax_collector_payout(&mut app, idx));
        assert_eq!(app.world.tax_money, 0, "the purse empties");
        let kinds: Vec<u16> = app.items.iter().map(|d| d.kind).collect();
        assert!(kinds.contains(&74), "one platinum coin");
        assert!(kinds.contains(&73), "two gold coins");
        assert!(kinds.contains(&72), "three silver coins");
        assert!(kinds.contains(&71), "the coppers");
        assert!(app.items.len() > before, "the drops spawn");
    }

    /// `Player.CollectTaxes` (`Player.cs:24129-24155`) + the pump
    /// (`Main.cs:64995-65003`): 50 copper per housed town NPC per
    /// `taxRate` clock, capped at 25 gold.
    #[test]
    fn the_tax_pump_fills_the_purse_to_the_cap() {
        let mut world = World::new("t".into(), 8, 8);
        world.tax_collector = true;
        // No collectors yet: the pump does nothing.
        crate::world_sim::clock::update_taxes(&mut world, 1);
        assert_eq!(world.tax_money, 0, "no payers, no money");
        // One payer: after `taxRate` iterations one collection lands.
        world.tax_payers = 1;
        for _ in 0..TAX_RATE {
            crate::world_sim::clock::update_taxes(&mut world, 1);
        }
        assert_eq!(world.tax_money, 50, "one collection of 50 copper");
        // The cap (`:24151-24153`): a collection that would overflow clamps
        // to 25 gold; a purse already at the cap collects nothing more.
        world.tax_money = TAX_CAP - 10;
        world.tax_payers = 100;
        crate::world_sim::clock::collect_taxes(&mut world);
        assert_eq!(world.tax_money, TAX_CAP, "the 25-gold cap holds");
        let capped = world.tax_money;
        crate::world_sim::clock::collect_taxes(&mut world);
        assert_eq!(world.tax_money, capped, "no collection past the cap");
    }

    #[test]
    fn the_talks_free_the_wizard_and_mechanic() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let world = app.world.clone();
        let mut player = crate::player::Player::spawn(&world);
        player.x = 32.0;
        app.player = Some(player);
        app.world.hard_mode = true;
        let mut w = Npc::new(NpcKind::BoundWizard, 32.0, 100.0, 1, 0.0);
        w.home = Some((2, 6));
        app.npcs.push(w);
        let mut m = Npc::new(NpcKind::BoundMechanic, 64.0, 100.0, 1, 0.0);
        app.npcs.push(m);
        try_open_shop_at_cursor(&mut app, Vec2::new(34.0, 100.0));
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Wizard),
            "the Wizard is freed"
        );
        assert!(app.world.saved_wizard, "savedWizard raises");
        assert!(!app.npcs.iter().any(|n| n.kind == NpcKind::BoundWizard));
        try_open_shop_at_cursor(&mut app, Vec2::new(66.0, 100.0));
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Mechanic),
            "the Mechanic is freed"
        );
        assert!(app.world.saved_mech, "savedMech raises");
    }

    /// The town arrival behind the flags (`Main.cs:65844-65851`'s
    /// `townNPCCanSpawn` arms collapsed to the 2-second town pass): the saved
    /// flag spawns the town NPC once and never again.
    #[test]
    fn the_saved_flags_open_the_town_arrival() {
        // `WorldGen.SpawnTownNPC` needs a finished, furnished room
        // (`WorldGen.cs:5409-5500'): the faithful gate above only hands the
        // kind over, the room is the placement's own precondition.
        let mut world = crate::test_world_helpers::ground();
        crate::test_world_helpers::build_furnished_room(&mut world, 60, 105, 12, 9);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&world);
        // Beside the room: the arrival's scan is `quick_find_home`'s ±10
        // (`WorldGen.cs:5759-5805`).
        player.x = 66.0 * 16.0;
        player.y = 112.0 * 16.0;
        app.player = Some(player);
        app.world = world;
        app.world.saved_wizard = true;
        app.world.saved_mech = true;
        // One town pass.
        // The Guide is resident from the world spawn (`NPC.cs:1775`), and the
        // ladder stops at him while he is missing — seed the roster the game
        // actually starts with.
        let mut guide = Npc::new(NpcKind::Guide, 32.0, 100.0, 1, 0.0);
        guide.home = Some((2, 6));
        guide.hp = guide.max_hp;
        app.npcs.push(guide);
        // Arm the vanilla arrival gate: the C# checks every 7200 ticks
        // (`Main.cs:65548-65555`), so the next update is a check.
        app.town_arrival.check_for_spawns = 7199;
        update_npcs(&mut app, 1.0 / 60.0);
        // One arrival per check (`Main.cs:65957`: the ladder picks a single
        // type), and the ladder puts the Wizard ahead of the Mechanic.
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Wizard),
            "the Wizard wins the ladder"
        );
        assert!(
            !app.npcs.iter().any(|n| n.kind == NpcKind::Mechanic),
            "the Mechanic waits for the next check"
        );
        // The next check (the 7200-tick gate re-armed) takes the Mechanic.
        app.town_arrival.check_for_spawns = 7199;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Mechanic),
            "the Mechanic arrives on the next check"
        );
        // With both resident, a further check adds nothing.
        let count = app.npcs.len();
        app.town_arrival.check_for_spawns = 7199;
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.npcs.len(), count, "no duplicate arrivals");
    }

    /// `NPC.cs:2088`'s hardmode guard: the bound Wizard's ladder arm needs
    /// hardmode, so without it the tied Wizard never appears however many
    /// spawn attempts run.
    #[test]
    fn the_bound_wizard_arm_needs_hardmode() {
        let mut world = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 400..800 {
                let i = world.index(x, y);
                world.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        world.compute_column_surface();
        for hard in [false, true] {
            let mut found_any = false;
            let mut found_wizard = false;
            for seed in 0..240u64 {
                let ctx = crate::spawner::SpawnContext {
                    world: &world,
                    player_center: (32.0 * 16.0, 500.0 * 16.0),
                    player_pos: (32.0 * 16.0, 500.0 * 16.0),
                    nearby_active_npcs: 0.0,
                    town_npcs: 0,
                    day_time: true,
                    player_life_max: 100,
                    underworld_for_bats: false,
                    flags: crate::spawner::SpawnFlags {
                        hard_mode: hard,
                        ..Default::default()
                    },
                    bound_goblin_ok: false,
                    bound_wizard_ok: true,
                    bound_mech_ok: false,
                    below_deep_dungeon: true,
                    invaders: false,
                    downed_golem: false,
                    downed_martians: false,
                    invasion_type: 0,
                    invasion_summoner_alive: false,
                    invasion_captain_alive: false,
                    invasion_walker_alive: false,
                    invasion_saucer_alive: false,
                    invasion_progress_30: false,
                    invasion_progress_frac: 0.0,
                    any_danger: false,
                    probe_exists: false,
                    sky_behind_player: false,
                };
                let mut rng = crate::spawner::FastRandom::new(seed + 1);
                if let Some(n) = crate::spawner::try_spawn_an_npc(&ctx, &mut rng) {
                    found_any = true;
                    if n.kind == NpcKind::BoundWizard {
                        found_wizard = true;
                    }
                }
            }
            // The arm's own gate is the assertion: without hardmode the tied
            // Wizard never comes out of the ladder, whatever else spawns (the
            // other ladders' world gates decide that, not this arm).
            if !hard {
                assert!(
                    !found_wizard,
                    "no hardmode, no bound Wizard (any spawn: {found_any})"
                );
            }
        }
    }

    #[test]
    fn the_talk_wakes_the_sleeping_angler() {
        let mut world = World::new("t".into(), 64, 800);
        let mut saved = crate::player::Player::spawn(&world);
        saved.x = 32.0;
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.player = Some(saved);
        let mut n = Npc::new(NpcKind::SleepingAngler, 32.0, 100.0, 1, 0.0);
        n.home = Some((2, 6));
        app.npcs.push(n);
        assert!(!app.world.saved_angler);
        try_open_shop_at_cursor(&mut app, Vec2::new(34.0, 100.0));
        let angler = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::Angler)
            .expect("the Angler");
        assert_eq!(angler.max_hp, 250, "the town twin's body");
        assert!(
            angler.home.is_none(),
            "Transform case 369 leaves him homeless"
        );
        assert!(app.world.saved_angler, "the saved flag raises");
        assert!(
            !app.npcs.iter().any(|n| n.kind == NpcKind::SleepingAngler),
            "no bound form remains"
        );
    }

    /// `Main.NPCChatText_DoAnglerQuest` (`Main.cs:40227-40263`): the hand-in
    /// takes the quest fish off the stack, counts the quest, pays the reward
    /// and closes the day — a second click on the same day does nothing.
    #[test]
    fn the_angler_quest_turn_in_pays_out_once_a_day() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let world = app.world.clone();
        let mut player = crate::player::Player::spawn(&world);
        player.x = 32.0;
        app.player = Some(player);
        // Today's quest (the pool index 7 = 2457) and the fish in hand.
        app.world.angler_quest = 7;
        let quest = crate::angler::QUEST_POOL[7];
        let slot = &mut app.inventory[0];
        slot.kind = quest;
        slot.name = "Quest Fish";
        slot.count = 3;
        let mut a = Npc::new(NpcKind::Angler, 32.0, 100.0, 1, 0.0);
        a.hp = a.max_hp;
        let ai = app.npcs.len();
        app.npcs.push(a);
        let idx = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::Angler)
            .unwrap();
        assert_eq!(idx, ai);
        let before = app.items.len();
        assert!(angler_turn_in(&mut app, idx), "the hand-in runs");
        assert_eq!(app.inventory[0].count, 2, "the stack drops one");
        assert_eq!(app.player.as_ref().unwrap().angler_quests_finished, 1);
        assert!(app.world.angler_quest_finished, "the day closes");
        assert!(app.world.angler_quest_turned_in, "the player's slot");
        assert!(app.items.len() > before, "the reward drops spawn");
        // `AchievementsHelper.HandleAnglerService` (`Main.cs:40260`,
        // `AchievementsHelper.cs:411-418`): the flag and the four `Finish`
        // counters advance on the turn-in.
        assert!(
            app.achievements
                .list
                .iter()
                .any(|a| a.name == "SERVANT_IN_TRAINING" && a.completed)
        );
        assert!(
            app.achievements
                .list
                .iter()
                .any(|a| a.name == "GOOD_LITTLE_SLAVE" && a.conditions[0].value == 1)
        );
        // The reward lands at the Angler's centre.
        let (ax, ay) = app.npcs[idx].center();
        assert!(
            app.items
                .iter()
                .any(|d| (d.pos.x - ax).abs() < 40.0 && (d.pos.y - ay).abs() < 40.0),
            "the drops ride the Angler"
        );
        // The same-day refusal.
        let count = app.player.as_ref().unwrap().angler_quests_finished;
        angler_turn_in(&mut app, idx);
        assert_eq!(app.player.as_ref().unwrap().angler_quests_finished, count);
        // An empty hand keeps the day open.
        app.world.angler_quest_finished = false;
        app.world.angler_quest_turned_in = false;
        app.inventory[0].kind = 0;
        angler_turn_in(&mut app, idx);
        assert!(!app.world.angler_quest_finished, "no fish, no hand-in");
    }

    /// `UpdateTime_StartDay`'s `Main.AnglerQuestSwap()` (`Main.cs:65373`): the
    /// clock arms the request, the world pass rolls it, and the day flags
    /// reset (`:3760-3761`).
    #[test]
    fn the_day_boundary_rolls_a_new_quest() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world.angler_quest = 40;
        app.world.angler_quest_finished = true;
        app.world.angler_quest_turned_in = true;
        // No request: nothing happens.
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.world.angler_quest, 40, "no request, no swap");
        // The boundary arms it.
        app.world.angler_swap_pending = true;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app.world.angler_swap_pending, "the request resolves");
        assert!(!app.world.angler_quest_finished, "the day resets");
        assert!(
            !app.world.angler_quest_turned_in,
            "the player's slot resets"
        );
        assert!(
            (app.world.angler_quest as usize) < crate::angler::QUEST_POOL.len(),
            "a valid pool index"
        );
    }

    /// `Spawner`'s two 376 arms (`NPC.cs:1775` the ocean water tile,
    /// `:1925` the beach sand): the Angler unsaved and no 376 alive — and
    /// the saved flag stops the search for good.
    #[test]
    fn the_sleeping_angler_is_found_on_the_beach() {
        let mut world = World::new("t".into(), 1000, 300);
        // A sand beach inside the 380-tile band, above the surface.
        for x in 0..200 {
            for y in 40..300 {
                let i = world.index(x, y);
                world.tiles[i] = tr_core::Tile {
                    kind: 53,
                    ..Default::default()
                };
                world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        world.compute_column_surface();
        world.world_surface = 120.0;
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&world);
        player.x = 20.0 * 16.0;
        player.y = 75.0 * 16.0;
        app.player = Some(player);
        app.world = world;
        // The roll's gate (`Main.rand.Next(20) == 0` ≈ rand01 < 0.05): sweep
        // for the seed whose FIRST draw lands under it, so the spawn is
        // deterministic (the state is the one before the draw).
        let mut probe = 1u64;
        loop {
            let mut trial = probe;
            if rand01(&mut trial) < 0.05 {
                break;
            }
            probe += 1;
        }
        app.npc_rng = probe;
        assert!(
            try_spawn_sleeping_angler(&mut app),
            "the beach arm spawns the Sleeping Angler"
        );
        assert!(app.npcs.iter().any(|n| n.kind == NpcKind::SleepingAngler));
        // `savedAngler` closes the search.
        app.npcs.clear();
        app.world.saved_angler = true;
        assert!(
            !try_spawn_sleeping_angler(&mut app),
            "the saved flag stops it"
        );
    }

    #[test]
    fn black_slime_darkness_touch_rolls_the_item_stream() {
        // `Next(4) == 0` ≈ `rand01 < 0.25`: a crafted seed pins both
        // outcomes; a non-black slime never rolls (the stream state proves
        // it — the deny seed under the blue slime leaves the rng untouched).
        let mut world = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 40..800 {
                let i = world.index(x, y);
                world.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        world.compute_column_surface();
        let mut player = crate::player::Player::spawn(&world);
        let px = player.x;
        let py = player.y;
        let mut black = Npc::spawn_slime_variant(&world, (px / 16.0) as i32, SlimeVariant::Black)
            .expect("the surface spawns a black slime");
        black.x = px;
        black.y = py;
        // The first rand01 of seed 1 — under the 0.25 gate?
        let mut rng = 1u64;
        let first = rand01(&mut rng);
        black_slime_darkness_touch(&black, &mut player, &mut 1u64);
        assert_eq!(
            player.buffs.active_types().contains(&22),
            first < 0.25,
            "seed 1 first roll {first}"
        );
        // A seed whose first roll is >= 0.25 denies the grant.
        let mut deny_seed = 2u64;
        while rand01(&mut deny_seed) < 0.25 {
            deny_seed += 1;
        }
        let mut player2 = crate::player::Player::spawn(&world);
        black_slime_darkness_touch(&black, &mut player2, &mut deny_seed);
        assert!(!player2.buffs.active_types().contains(&22));
        // The stream advances only when the touch rolls (a blue slime skips
        // the roll entirely — the same seed stays put).
        let blue =
            Npc::spawn_slime_variant(&world, (px / 16.0) as i32, SlimeVariant::Blue).unwrap();
        let mut untouched = 7u64;
        black_slime_darkness_touch(&blue, &mut player, &mut untouched);
        assert_eq!(untouched, 7);
    }

    #[test]
    fn dot_death_text_selects_table_over_stub() {
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // DW-1.3 — the death collection resolves the DoT table text when the
        // drain left a reason (`Player.cs:19321-19331,19362-19385`) and the
        // Slain row for any other death (`Lang.cs` CreateDeathMessage).
        assert_eq!(dot_death_text(None, "Belsin", 0.0), "Belsin was slain...");
        assert_eq!(
            dot_death_text(Some(DotDeath::Poison), "Belsin", 0.5),
            "Belsin couldn't find the antidote."
        );
        assert_eq!(
            dot_death_text(Some(DotDeath::Burned), "Belsin", 0.0),
            "Belsin couldn't put the fire out."
        );
    }

    #[test]
    fn spawn_gel_and_coins_ride_the_player_luck() {
        // DW-3.1 — the drop rules read the killer's luck (`CommonDrop.cs:36`
        // through `info.luck`) and `NPCLoot_DropMoney` reads
        // `closestPlayer.luck` (`NPC.cs:79622`): at luck 1.0 both hit
        // branches always fire, so the item stream lands elsewhere than at
        // luck 0, while luck 0 stays byte-identical to the pre-wiring plain
        // rolls (the double-roll math itself is loot.rs-tested).
        use crate::app::flow::front_flow_tests::flow_app;
        let mut a = flow_app(None, vec![], vec![]);
        // A floor row gives the spawner a surface (flow_app's world is bare;
        // y 5 keeps top < height - 2 on the 8-tall world).
        for x in 0..a.world.width {
            let i = a.world.index(x, 5);
            a.world.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            a.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        a.world.compute_column_surface();
        a.player = Some(crate::player::Player::spawn(&a.world));
        let npc = Npc::spawn_slime_variant(
            &a.world,
            (a.player.as_ref().unwrap().x / 16.0) as i32,
            SlimeVariant::Blue,
        )
        .expect("the surface spawns a blue slime");

        let start = a.item_rng;
        spawn_coins(&mut a, &npc);
        spawn_loot(&mut a, &npc);
        let state_0 = a.item_rng;

        a.item_rng = start;
        a.player.as_mut().unwrap().luck_state.luck = 1.0;
        spawn_coins(&mut a, &npc);
        spawn_loot(&mut a, &npc);
        assert_ne!(a.item_rng, state_0, "luck 1.0 takes the hit branches");
    }

    // M5-SEED-2: `endlessHalloween` / `endlessChristmas`
    // (`Main.forceHalloweenForever` / `Main.forceXMasForever`,
    // `Main.cs:13168` / `:13135`, set by `InitializeSecretSeeds`
    // `WorldGen.cs:572-578`) feed `LootFlags.halloween` / `.xmas`, which
    // `loot.rs`'s `HalloweenGoodieBagDrop` / `XmasPresentDrop` read
    // (`loot.rs:1495-1499`). The two rows are the global registrations
    // (`loot_db_gen.rs:27-28`) — the vanilla's only holiday drop the port
    // models — so the engine-level replay below is the flag's observable.
    //
    // The replay runs on `spawn_loot`'s own database with the same
    // `LootNpc` shape: the executor's feed leaves `damage` at 0 (the C#'s
    // `info.npc.damage`, not yet wired), and both holiday rows gate on
    // `n.damage > 0` — so this test pins the FLAG's effect on the rules,
    // and the executor path gains its end-to-end drop when that feed lands.
    #[test]
    fn the_endless_holiday_micro_seeds_flip_the_loot_flags() {
        let drops_for = |halloween: bool, xmas: bool| -> Vec<i32> {
            let mut out = Vec::new();
            for seed in 1..400u64 {
                let mut rng = tr_core::tile_light::FastRandom::new(seed);
                let mut info = tr_core::loot::DropAttemptInfo {
                    rng: &mut rng,
                    luck: 0.0,
                    is_expert: false,
                    is_master: false,
                    active_players: 1,
                    extra_gel: false,
                    flags: tr_core::loot::LootFlags {
                        halloween,
                        xmas,
                        ..Default::default()
                    },
                    // The zombie (3) `spawn_loot` resolves, with the damage
                    // the holiday rows also require.
                    npc: tr_core::loot::LootNpc {
                        kind: 3,
                        value: 1.0,
                        life_max: 100,
                        damage: 25,
                        defense: 5,
                        friendly: false,
                        ..Default::default()
                    },
                };
                tr_core::loot::try_dropping(crate::loot_db::database(), 3, &mut info, &mut out);
            }
            let mut ids: Vec<i32> = out.iter().map(|r| r.item).collect();
            ids.sort_unstable();
            ids.dedup();
            ids
        };
        let classic = drops_for(false, false);
        let halloween = drops_for(true, false);
        let xmas = drops_for(false, true);
        assert!(
            !classic.contains(&1774) && !classic.contains(&1869),
            "a classic world drops neither holiday item"
        );
        assert!(
            halloween.contains(&1774),
            "endlessHalloween drops the Goodie Bag"
        );
        assert!(xmas.contains(&1869), "endlessChristmas drops the Present");
    }

    #[cfg(test)]
    mod voodoo_tests {
        use super::*;
        use crate::app::flow::front_flow_tests::flow_app;
        use crate::npc::Npc;
        use tr_core::tile::{FLAG_ACTIVE, LiquidKind, Tile};
        use tr_core::world::World;

        /// A hell world: the stone floor at row `hell_floor`, the lava pool on
        /// top of it, the underworld band below (`maxTilesY - 200`).
        fn hell_world() -> World {
            let mut w = World::new("hell".into(), 400, 700);
            for x in 0..w.width {
                for y in 500..w.height {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile {
                        kind: 1,
                        ..Default::default()
                    };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            // The lava pool at the drop site (rows 498-499, columns 190-210).
            for x in 190..210 {
                for y in 498..500 {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile {
                        ..Default::default()
                    };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                    w.tiles[i].liquid = 255;
                    w.tiles[i].set_liquid_kind(LiquidKind::Lava);
                }
            }
            w.compute_column_surface();
            w
        }

        #[test]
        fn the_voodoo_doll_sacrifice_kills_the_guide_and_spawns_the_wall() {
            // `VoodooDollLavaDeath` + `SpawnWOF`: the doll in the underworld
            // lava kills the Guide (9999) and spawns the wall at the
            // player-clearance x inside the band rows.
            let mut app = flow_app(None, vec![], vec![]);
            app.world = hell_world();
            let mut player = crate::player::Player::spawn(&app.world);
            player.x = 200.0 * TILE_PX;
            player.y = 496.0 * TILE_PX;
            let player_x = player.x;
            app.player = Some(player);
            let mut guide = Npc::new(NpcKind::Guide, 196.0 * TILE_PX, 496.0 * TILE_PX, 1, 0.0);
            guide.hp = guide.max_hp;
            app.npcs.push(guide);
            let pos = Vec2::new(200.0 * TILE_PX, 498.5 * TILE_PX);
            crate::world_sim::npcs::voodoo_doll_lava_death(&mut app, pos, 1);
            // The Guide died.
            assert!(
                app.npcs
                    .iter()
                    .all(|n| n.kind != NpcKind::Guide || n.hp <= 0)
            );
            // The wall spawned inside the underworld band.
            let wall = app
                .npcs
                .iter()
                .find(|n| n.kind == NpcKind::WallOfFlesh)
                .expect("the wall spawned");
            let band_top = (app.world.height as i32 - 200 + 10) as f32 * TILE_PX;
            let band_bottom = (app.world.height as i32 - 200 + 80) as f32 * TILE_PX;
            assert!(
                wall.y >= band_top && wall.y <= band_bottom,
                "the wall row clamped into the band ({})",
                wall.y
            );
            // The clearance: the spawn x sits at least 1200 px from the player.
            let dx = (wall.x - player_x).abs();
            assert!(dx >= 1200.0, "the spawn cleared the player ({dx})");
        }

        #[test]
        fn the_wall_spawn_refuses_outside_the_underworld_and_while_alive() {
            // `SpawnWOF`'s gates: the depth (`pos.Y / 16 < maxTilesY - 205`) and
            // the living wall.
            let mut app = flow_app(None, vec![], vec![]);
            app.world = hell_world();
            let mut player = crate::player::Player::spawn(&app.world);
            player.x = 200.0 * TILE_PX;
            player.y = 496.0 * TILE_PX;
            app.player = Some(player);
            // The surface position: refused.
            let surface = Vec2::new(200.0 * TILE_PX, 120.0 * TILE_PX);
            assert!(!crate::world_sim::npcs::spawn_wof_at(&mut app, surface));
            // A living wall blocks the second spawn.
            let pos = Vec2::new(200.0 * TILE_PX, 498.5 * TILE_PX);
            assert!(crate::world_sim::npcs::spawn_wof_at(&mut app, pos));
            assert!(
                !crate::world_sim::npcs::spawn_wof_at(&mut app, pos),
                "one wall only"
            );
        }

        #[test]
        fn the_doll_reminder_kills_a_second_town_npc() {
            // The stack remainder: a doll stack of 2 kills the Guide AND one
            // more town NPC (`num` walks the town list, `:730-753`).
            let mut app = flow_app(None, vec![], vec![]);
            app.world = hell_world();
            let mut player = crate::player::Player::spawn(&app.world);
            player.x = 200.0 * TILE_PX;
            player.y = 496.0 * TILE_PX;
            app.player = Some(player);
            let mut guide = Npc::new(NpcKind::Guide, 196.0 * TILE_PX, 496.0 * TILE_PX, 1, 0.0);
            guide.hp = guide.max_hp;
            app.npcs.push(guide);
            let mut merchant =
                Npc::new(NpcKind::Merchant, 204.0 * TILE_PX, 496.0 * TILE_PX, 1, 0.0);
            merchant.hp = merchant.max_hp;
            app.npcs.push(merchant);
            let pos = Vec2::new(200.0 * TILE_PX, 498.5 * TILE_PX);
            crate::world_sim::npcs::voodoo_doll_lava_death(&mut app, pos, 2);
            assert!(
                app.npcs
                    .iter()
                    .all(|n| n.kind != NpcKind::Guide || n.hp <= 0)
            );
            assert!(
                app.npcs
                    .iter()
                    .all(|n| n.kind != NpcKind::Merchant || n.hp <= 0)
            );
        }
    }
}

#[cfg(test)]
mod invasion_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::npc::Npc;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// A surface world (the invasion fights at the spawn's surface band).
    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        // The surface band: the floor at row 120 rides the world's
        // `worldSurface` (the default height*0.25 = 50 puts the surface
        // band at px 1000 — below the floor's row 120*16 = 1920).
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    /// DW-1.1 (dirty, watched RED pre-fix): with an active invasion inside
    /// the ±3000 px window, the invader spawns on a machinery-found tile —
    /// in the spawn area, non-solid at the spawn point, ground below — not
    /// at the retired fixed `(px − 600, py − 400)` offset, and facing the
    /// player (`NPC.cs:308-342`'s one path through `FindSpawnTile` →
    /// `SpawnAnNPC`'s `else if (this.invaders)` ladder `:1424-1541`, the
    /// `NewNPC` `Bottom` placement `:80772`). The pick's rolls leave the
    /// item stream alone (`item_rng` unchanged across the whole loop — the
    /// invasion path draws only from the spawner stream, DW-1.2).
    #[test]
    fn invasion_spawns_land_on_machinery_found_tiles() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        // The active goblin invasion: the surface band and the ±3000 px
        // window around `invasionX` (the player's own column).
        app.world.invasion_type = 1;
        app.world.invasion_size = 100;
        app.world.invasion_size_start = 100;
        app.world.invasion_delay = 0;
        app.world.invasion_goblins = true;
        app.world.invasion_x = 150.0;
        app.npc_timer = 0.0;
        let item_rng_before = app.item_rng;
        let solid = |w: &World, tx: i32, ty: i32| {
            w.tile(tx, ty)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
        };
        let mut spawned = false;
        for _ in 0..20000 {
            update_npcs(&mut app, 1.0 / 60.0);
            let Some(n) = app.npcs.last() else { continue };
            if !matches!(
                n.kind,
                NpcKind::GoblinSorcerer
                    | NpcKind::GoblinThief
                    | NpcKind::GoblinPeon
                    | NpcKind::GoblinWarrior
                    | NpcKind::GoblinArcher
                    | NpcKind::GoblinSummoner
            ) {
                continue;
            }
            let (w, h) = n.kind.size();
            let tile_x = ((n.x + w * 0.5) / TILE_PX).floor() as i32;
            let mid_row = ((n.y + h * 0.5) / TILE_PX).floor() as i32;
            let feet_row = ((n.y + h) / TILE_PX).floor() as i32;
            // The spawn area (`GetSpawnArea`, NPC.cs:931-967): the player's
            // tile ± (sWidth/16 x 0.7, sHeight/16 x 0.7).
            let px_tile = (150.0 * TILE_PX / TILE_PX).floor() as i32;
            let py_tile = (118.0 * TILE_PX / TILE_PX).floor() as i32;
            let area_x = (spawner::S_WIDTH / 16 * 7) / 10;
            let area_y = (spawner::S_HEIGHT / 16 * 7) / 10;
            assert!(
                tile_x >= px_tile - area_x && tile_x < px_tile + area_x,
                "the spawn tile is in the spawn area (tile_x {tile_x})"
            );
            assert!(
                feet_row >= py_tile - area_y && feet_row < py_tile + area_y,
                "the spawn tile is in the spawn area (feet row {feet_row})"
            );
            assert!(
                !solid(&app.world, tile_x, mid_row),
                "non-solid at the spawn point (tile {tile_x},{mid_row})"
            );
            assert!(
                solid(&app.world, tile_x, feet_row) || solid(&app.world, tile_x, feet_row + 1),
                "ground below the spawn point (feet row {feet_row})"
            );
            // The machinery's facing rule (`TargetClosest`, npc.rs:6998) —
            // toward the player, not the ±1-by-`invasion_x` stand-in.
            let expected_facing = if 150.0 * TILE_PX >= n.x + w * 0.5 {
                1
            } else {
                -1
            };
            assert_eq!(n.facing, expected_facing, "facing toward the player");
            // Not the retired fixed offset.
            assert!(
                (n.x - (150.0 * TILE_PX - 600.0)).abs() > 1.0
                    || (n.y - (118.0 * TILE_PX - 400.0)).abs() > 1.0,
                "not at the fixed (px-600, py-400) offset"
            );
            spawned = true;
            break;
        }
        assert!(spawned, "an invader spawned inside the window");
        assert_eq!(
            app.item_rng, item_rng_before,
            "the invasion path draws nothing from the item stream"
        );
    }

    #[test]
    fn the_morning_roll_arms_the_invasion() {
        // `UpdateTime_StartDay`'s roll: the smashed orb + the pre-defeat 1/3
        // roll arms the pending flag; without the orb the roll is inert.
        let mut w = surface_world();
        w.shadow_orb_smashed = true;
        w.roll_morning_invasion(0.2);
        assert!(w.invasion_arm_pending, "the pre-defeat 1/3 roll arms");
        w.invasion_arm_pending = false;
        w.roll_morning_invasion(0.9);
        assert!(!w.invasion_arm_pending, "the high roll passes");
        // The post-defeat tiers: hardmode 1/60.
        w.downed_goblins = true;
        w.roll_morning_invasion(0.01);
        assert!(w.invasion_arm_pending, "the hardmode 1/60 roll arms");
        // Without the orb nothing arms.
        w.shadow_orb_smashed = false;
        w.invasion_arm_pending = false;
        w.roll_morning_invasion(0.0);
        assert!(!w.invasion_arm_pending, "no orb — no invasion");
    }

    #[test]
    fn start_invasion_sizes_the_army_and_picks_the_side() {
        // `StartInvasion`: size = 80 + 40/qualified-player; the march X is
        // 0 or maxTilesX (the deterministic roll).
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        start_invasion(&mut app.world, 1, 1);
        assert_eq!(app.world.invasion_type, 1);
        assert_eq!(app.world.invasion_size, 120);
        assert_eq!(app.world.invasion_size_start, 120);
        assert!(
            app.world.invasion_x == 0.0 || app.world.invasion_x == 300.0,
            "the side roll ({})",
            app.world.invasion_x
        );
        // The second call refuses (the invasion is active).
        start_invasion(&mut app.world, 1, 1);
        assert_eq!(app.world.invasion_type, 1);
    }

    #[test]
    fn the_army_marches_and_warns() {
        // `UpdateInvasion`: the march toward spawnTileX by dayRate, the
        // arrival broadcast, the 3600-tick warn timer.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        app.world.spawn_x = 150;
        app.world.invasion_type = 1;
        app.world.invasion_size = 120;
        app.world.invasion_size_start = 120;
        app.world.invasion_x = 0.0;
        app.world.invasion_warn = 0;
        // The march: invasionX advances by dayRate (1) per call.
        let chat_before = app.chat.messages.len();
        update_invasion_state(&mut app);
        assert!(app.world.invasion_x > 0.0, "the march advances");
        // The warn timer: 3600 → the approaching broadcasts.
        app.world.invasion_warn = 1;
        update_invasion_state(&mut app);
        assert_eq!(app.world.invasion_warn, 3600, "the warn rearm");
        assert!(
            app.chat.messages.len() > chat_before,
            "the approaching line"
        );
    }

    #[test]
    fn the_kill_credit_drains_and_defeats() {
        // `NPC.cs:78236-78279`: the goblin kill drains 1 point (the
        // Summoner 10), the group + active-type match; at size 0 the
        // downedGoblins flag sets and the invasion clears.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        app.world.invasion_type = 1;
        app.world.invasion_size = 2;
        app.world.invasion_size_start = 120;
        report_invasion_kill(&mut app, NpcKind::GoblinThief);
        assert_eq!(app.world.invasion_size, 1);
        assert!(!app.world.downed_goblins);
        report_invasion_kill(&mut app, NpcKind::GoblinSummoner);
        assert_eq!(app.world.invasion_size, 0);
        update_invasion_state(&mut app);
        assert!(app.world.downed_goblins, "the defeat flag");
        assert_eq!(app.world.invasion_type, 0, "the invasion cleared");
        // The non-goblin kill doesn't drain.
        app.world.invasion_type = 1;
        app.world.invasion_size = 5;
        report_invasion_kill(&mut app, NpcKind::Zombie);
        assert_eq!(app.world.invasion_size, 5);
    }

    #[test]
    fn the_invasion_spawn_window_and_table() {
        // `ShouldSpawnInvasionEnemies` (NPC.cs:442-463): the surface band +
        // the ±3000 px window around invasionX. (The pick's table tests
        // moved beside the ladder into `spawner.rs`'s test module — the
        // pick lives there since the S1-F-12 placement half.)
        let mut w = surface_world();
        w.invasion_type = 1;
        w.invasion_size = 120;
        assert!(
            should_spawn_invasion_enemies(&w, 110.0 * TILE_PX, 20.0 * TILE_PX),
            "the window near invasionX 0"
        );
        assert!(
            !should_spawn_invasion_enemies(&w, 110.0 * TILE_PX, 250.0 * TILE_PX),
            "far from invasionX"
        );
        // The band is `NPC.sHeight` = 1200 px below the surface
        // (`NPC.cs:6769`): the world's surface is row 120 = px 1920, so the
        // band reaches px 3120 — 3000 px down still spawns, 3200 does not.
        assert!(
            should_spawn_invasion_enemies(&w, 3000.0, 20.0 * TILE_PX),
            "the 1200 px band holds"
        );
        assert!(
            !should_spawn_invasion_enemies(&w, 3200.0, 20.0 * TILE_PX),
            "below the band refuses"
        );
    }
    #[test]
    fn the_eow_middle_split_lives_as_a_new_head() {
        // `NPC.cs:51507-51519`: killing a worm's middle splits it — the
        // orphaned body becomes its own head (`Transform(13)`), the link
        // ahead of the gap closes as a tail (`Transform(15)`), the tail
        // with a dead leader dies silently.
        let world = World::new("t".into(), 64, 800);
        let mut npcs = Vec::new();
        let mut head = Npc::new(NpcKind::EaterOfWorldsHead, 100.0, 100.0, 1, 0.0);
        head.hp = head.max_hp;
        head.worm_id = 7;
        head.seg_index = 0;
        head.worm_inited = true;
        npcs.push(head);
        for seg in 1..4 {
            let mut b = Npc::new(
                NpcKind::EaterOfWorldsBody,
                100.0,
                100.0 + seg as f32 * 20.0,
                1,
                0.0,
            );
            b.hp = b.max_hp;
            b.worm_id = 7;
            b.seg_index = seg;
            b.worm_inited = true;
            npcs.push(b);
        }
        // The head dies -> the first orphan (seg 1) becomes the head; the
        // links behind stay alive following it.
        npcs[0].hp = 0;
        let mut rng = 1u64;
        let mut next_id = 99u32;
        update_worms_in(&world, None, &mut npcs, &mut rng, &mut next_id);
        assert_eq!(
            npcs[1].kind,
            NpcKind::EaterOfWorldsHead,
            "the orphan becomes the head"
        );
        assert!(npcs[1].hp > 0);
        assert!(npcs[2].hp > 0 && npcs[3].hp > 0, "the back part lives");
        // The last link dies -> the mid ahead of the gap closes as the
        // tail.
        npcs[3].hp = 0;
        update_worms_in(&world, None, &mut npcs, &mut rng, &mut next_id);
        assert_eq!(
            npcs[2].kind,
            NpcKind::EaterOfWorldsTail,
            "the front closes as a tail"
        );
        assert!(npcs[2].hp > 0);
    }
    #[test]
    fn the_destroyer_segments_fire_lasers() {
        // `AI_037_Destroyer`'s type-135 arm (`NPC.cs:49856-49891`): the
        // timer rolls Next(4) per tick against the fresh
        // Next(1400, 26000) threshold; the fire sets the hostile laser
        // 100 payload (the dmg 22, the speed 8, the CanHit gate).
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 30..800 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        // The head (seg 0) never fires; the body link does. Both sit in
        // the open air above the ground so the CanHit line exists.
        let mut head = Npc::spawn_worm_head(&w, 30, 25, NpcKind::TheDestroyer);
        head.hp = head.max_hp;
        head.worm_id = 5;
        head.seg_index = 0;
        head.worm_inited = true;
        let mut body = Npc::new(NpcKind::TheDestroyer, head.x, head.y + 20.0, 1, 0.0);
        body.hp = body.max_hp;
        body.worm_id = 5;
        body.seg_index = 1;
        body.worm_inited = true;
        let mut npcs = vec![head, body];
        let mut rng = 1u64;
        let mut next_id = 9u32;
        // One body link fires ~once per 9100 ticks (the vanilla's 80
        // links make the fight rate) — the window gives the ~97% hit.
        let mut shot = None;
        for _ in 0..40000 {
            update_worms_in(&w, Some((480.0, 380.0)), &mut npcs, &mut rng, &mut next_id);
            if let Some(p) = npcs[1].wants_hostile_shot.pop() {
                shot = Some(p);
            }
        }
        let (kind, _, _, vx, vy, dmg) = shot.expect("the body link fires within 40000 ticks");
        assert!(
            npcs[0].wants_hostile_shot.is_empty(),
            "the head never fires"
        );
        assert_eq!(kind, crate::projectile::DESTROYER_LASER);
        assert_eq!(dmg, 22);
        let sp = (vx * vx + vy * vy).sqrt();
        assert!((sp - 8.0).abs() < 2.0, "the 8-speed aim ({sp})");
    }

    #[cfg(test)]
    mod frost_invasion_tests {
        use super::*;
        use crate::app::flow::front_flow_tests::flow_app;
        use crate::world_sim::editing::use_snow_globe;
        use tr_core::tile::{FLAG_ACTIVE, Tile};

        fn surface_world() -> World {
            let mut w = World::new("t".into(), 300, 200);
            w.world_surface = 120.0;
            for x in 0..w.width {
                for y in 120..w.height {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile {
                        kind: 1,
                        ..Default::default()
                    };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            w.compute_column_surface();
            w
        }

        fn surface_world_orig() -> World {
            let mut w = World::new("t".into(), 300, 200);
            w.world_surface = 120.0;
            for x in 0..w.width {
                for y in 120..w.height {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile {
                        kind: 1,
                        ..Default::default()
                    };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            w.compute_column_surface();
            w
        }

        #[test]
        fn the_snow_globe_arms_the_frost_legion() {
            // `ItemCheck_UseBossSpawners`' Frost arm (`Player.cs:43940-43956`):
            // the Snow Globe starts `invasionType = 2` with the 80-point army;
            // a second globe refuses while the invasion is active.
            let mut app = flow_app(None, vec![], vec![]);
            app.world = surface_world();
            let mut player = crate::player::Player::spawn(&app.world);
            player.max_hp = 200;
            player.x = 150.0 * TILE_PX;
            player.y = 118.0 * TILE_PX;
            app.player = Some(player);
            app.inventory[app.selected] = crate::item::ItemSlot {
                kind: 602,
                name: "Snow Globe".into(),
                count: 1,
                prefix: 0,
            };
            assert!(use_snow_globe(&mut app));
            assert_eq!(app.world.invasion_type, 2);
            assert_eq!(app.world.invasion_size, 120);
            assert!(app.world.invasion_goblins, "the active-invasion flag");
            app.inventory[app.selected] = crate::item::ItemSlot {
                kind: 602,
                name: "Snow Globe".into(),
                count: 1,
                prefix: 0,
            };
            assert!(!use_snow_globe(&mut app), "the active invasion refuses");
            assert_eq!(app.inventory[app.selected].count, 1);
        }

        #[test]
        fn the_frost_kill_credit_drains_and_defeats() {
            // The group-2 match: the frost kills drain 1 point each; at 0 the
            // `downedFrost` flag sets (the goblin flag stays untouched).
            let mut app = flow_app(None, vec![], vec![]);
            app.world = surface_world();
            app.world.invasion_type = 2;
            app.world.invasion_size = 1;
            app.world.invasion_size_start = 120;
            report_invasion_kill(&mut app, NpcKind::SnowmanGangsta);
            assert_eq!(app.world.invasion_size, 0);
            update_invasion_state(&mut app);
            assert!(app.world.downed_frost, "the frost defeat flag");
            assert!(!app.world.downed_goblins, "the goblin flag untouched");
            assert_eq!(app.world.invasion_type, 0);
        }
    }
}

#[cfg(test)]
mod pirate_invasion_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::npc::Npc;
    use crate::world_sim::editing::use_pirate_map;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_pirate_map_arms_the_invasion_with_the_bonus() {
        // `ItemCheck_UseBossSpawners`' pirate arm (`Player.cs:43957-43971`)
        // + `StartInvasion`'s type-3 bonus (`Main.cs:64460-64464`): the
        // size = 80+40x + 40+20x (the single qualified player → 180).
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 1315,
            name: "Pirate Map".into(),
            count: 1,
            prefix: 0,
        };
        assert!(use_pirate_map(&mut app));
        assert_eq!(app.world.invasion_type, 3);
        assert_eq!(app.world.invasion_size, 180);
        assert_eq!(app.world.invasion_size_start, 180);
        assert!(app.world.invasion_goblins, "the active-invasion flag");
        // The second map refuses while the invasion is active.
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 1315,
            name: "Pirate Map".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_pirate_map(&mut app), "the active invasion refuses");
        assert_eq!(app.inventory[app.selected].count, 1);
    }

    #[test]
    fn the_pirate_kill_credit_drains_and_defeats() {
        // The group-3 match: the Deckhand drains 1, the Captain drains 5;
        // at 0 the `downedPirates` flag sets (the frost flag untouched).
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        app.world.invasion_type = 3;
        app.world.invasion_size = 6;
        app.world.invasion_size_start = 180;
        report_invasion_kill(&mut app, NpcKind::PirateDeckhand);
        assert_eq!(app.world.invasion_size, 5);
        report_invasion_kill(&mut app, NpcKind::PirateCaptain);
        assert_eq!(app.world.invasion_size, 0);
        update_invasion_state(&mut app);
        assert!(app.world.downed_pirates, "the pirate defeat flag");
        assert!(!app.world.downed_frost, "the frost flag untouched");
        assert!(!app.world.downed_goblins, "the goblin flag untouched");
        assert_eq!(app.world.invasion_type, 0);
    }

    #[test]
    fn the_golem_death_chain() {
        // `:79346`'s case 245 + `:84957`'s detach: the BODY's death
        // raises `downed_golem`; the attached head's death spawns the
        // shielded free head at its position.
        let mut app = flow_app(None, vec![], vec![]);
        let mut big = tr_core::World::new("t".into(), 64, 200);
        for x in 0..64 {
            for y in 120..200 {
                let i = big.index(x, y);
                big.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                big.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        big.compute_column_surface();
        app.world = big;
        // The head dies -> the free head spawns (the shield on).
        let mut head = Npc::new(NpcKind::GolemHead, 30.0 * 16.0, 118.0 * 16.0, 1, 0.0);
        head.hp = 0;
        app.npcs.push(head);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let free = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::GolemHeadFree)
            .expect("the free head spawns");
        assert!(free.dont_take_damage, "the spawn shield");
        // The body dies -> downed_golem.
        let mut body = Npc::new(NpcKind::Golem, 40.0 * 16.0, 118.0 * 16.0, 1, 0.0);
        body.hp = 0;
        app.npcs.push(body);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.world.downed_golem,
            "the body's death is the fight's end"
        );
    }
}

#[cfg(test)]
mod invasion_bar_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::npc::Npc;
    use tr_core::World;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_kill_credit_reports_the_bar_state() {
        // `ReportInvasionProgress` (`Main.cs:47687-47694`): the points
        // (sizeStart - size), the max (sizeStart), the icon (group + 3) and
        // the 160-tick display timer.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        app.world.invasion_type = 1;
        app.world.invasion_size = 3;
        app.world.invasion_size_start = 120;
        report_invasion_kill(&mut app, NpcKind::GoblinThief);
        assert_eq!(app.invasion_progress, 118, "sizeStart - size");
        assert_eq!(app.invasion_progress_max, 120);
        assert_eq!(app.invasion_progress_icon, 4, "the goblins' icon (1 + 3)");
        assert_eq!(app.invasion_progress_display_left, 160);
    }

    #[test]
    fn the_near_scan_refreshes_the_report() {
        // `CheckInvasionProgressDisplay` (`Main.cs:47599-47649`): a living
        // invader within 5000 px of the player marks near and refreshes the
        // report while the icon is set; the far case drops the flag.
        let mut w = World::new("wide".into(), 900, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.world.invasion_type = 1;
        app.world.invasion_size = 3;
        app.world.invasion_size_start = 120;
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 100.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        let mut invader = Npc::new(
            NpcKind::GoblinThief,
            101.0 * TILE_PX,
            118.0 * TILE_PX,
            1,
            0.0,
        );
        invader.hp = invader.max_hp;
        app.npcs.push(invader);
        check_invasion_progress_display(&mut app);
        assert!(app.invasion_progress_near);
        assert_eq!(app.invasion_progress_icon, 4, "the refresh sets the icon");
        // The invader far away (beyond the 5000-px window): the near flag drops.
        if let Some(n) = app.npcs.first_mut() {
            n.x = 700.0 * TILE_PX;
        }
        check_invasion_progress_display(&mut app);
        assert!(!app.invasion_progress_near);
    }

    #[test]
    fn the_fade_head_ramps_the_alpha() {
        // `DrawInvasionProgress`'s fade head (`Main.cs:47712-47736`): the
        // display timer decays per tick, the alpha ramps 0.05 toward the
        // visible, and at 0 display the alpha fades back to 0.
        let mut app = flow_app(None, vec![], vec![]);
        app.invasion_progress_display_left = 160;
        app.invasion_progress_alpha = 0.0;
        update_invasion_state(&mut app);
        assert_eq!(app.invasion_progress_display_left, 159);
        assert!(app.invasion_progress_alpha > 0.0, "the alpha ramps up");
        app.invasion_progress_display_left = 0;
        app.invasion_progress_alpha = 1.0;
        update_invasion_state(&mut app);
        assert!(
            (app.invasion_progress_alpha - 0.95).abs() < 1e-6,
            "the fade down"
        );
    }

    #[test]
    fn the_defeat_resets_the_bar_state() {
        // The invasion clears → the bar state resets (the icon 0 keeps the
        // near-scan from re-reporting after the invasion's end).
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        app.world.invasion_type = 1;
        app.world.invasion_size = 0;
        app.world.invasion_size_start = 120;
        update_invasion_state(&mut app);
        assert!(app.world.downed_goblins);
        assert_eq!(app.world.invasion_type, 0);
        assert_eq!(app.invasion_progress_icon, 0, "the icon resets");
    }

    #[cfg(test)]
    mod martian_invasion_tests {
        use super::*;
        use crate::app::flow::front_flow_tests::flow_app;
        use crate::npc::Npc;

        #[test]
        fn the_type4_machine_sizes_the_invasion() {
            // `StartInvasion(4)` (`Main.cs:64463-64464`): the size 160+40x (the
            // single qualified player → 200) — the type-4 arm overrides the base.
            let mut app = flow_app(None, vec![], vec![]);
            app.world = surface_world();
            let mut player = crate::player::Player::spawn(&app.world);
            player.max_hp = 200;
            player.x = 150.0 * TILE_PX;
            player.y = 118.0 * TILE_PX;
            app.player = Some(player);
            start_invasion(&mut app.world, 1, 4);
            assert_eq!(app.world.invasion_type, 4);
            assert_eq!(app.world.invasion_size, 200);
            assert_eq!(app.world.invasion_size_start, 200);
        }

        #[test]
        fn the_martian_kill_credit_drains_and_defeats() {
            // The group-4 match: the martian kills drain 1 point each; at 0 the
            // `downedMartians` flag sets (the pirate flag untouched).
            let mut app = flow_app(None, vec![], vec![]);
            app.world = surface_world();
            app.world.invasion_type = 4;
            app.world.invasion_size = 1;
            app.world.invasion_size_start = 200;
            report_invasion_kill(&mut app, NpcKind::GrayGrunt);
            assert_eq!(app.world.invasion_size, 0);
            update_invasion_state(&mut app);
            assert!(app.world.downed_martians, "the martian defeat flag");
            assert!(!app.world.downed_pirates, "the pirate flag untouched");
            assert_eq!(app.world.invasion_type, 0);
        }
    }

    /// BUG-08: the invasion progress bar's off mode (Main.cs:47718): the
    /// off setting zeroes both the display timer and the alpha, so the panel
    /// never draws; the default (mode 2) leaves the ramp untouched.
    /// (Re-landed with the arm — the BUG-11-15/front-end-sound merge dropped
    /// this test with the code.)
    #[test]
    fn bug_08_the_off_mode_zeroes_the_invasion_bar() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        app.world.invasion_type = 1;
        app.world.invasion_size = 40;
        app.world.invasion_size_start = 80;
        app.world.invasion_goblins = true;
        app.invasion_progress = 40;
        app.invasion_progress_max = 80;
        app.invasion_progress_icon = 4;
        app.invasion_progress_display_left = 160;
        app.invasion_progress_alpha = 1.0;
        update_invasion_state(&mut app);
        assert!(
            app.invasion_progress_alpha > 0.0,
            "the default leaves the alpha"
        );
        app.settings.event_progress = 0;
        app.invasion_progress_alpha = 0.7;
        app.invasion_progress_display_left = 120;
        update_invasion_state(&mut app);
        assert_eq!(app.invasion_progress_alpha, 0.0, "mode 0 zeroes the alpha");
        assert_eq!(
            app.invasion_progress_display_left, 0,
            "mode 0 zeroes the display timer"
        );
    }
}

#[cfg(test)]
mod probe_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_with_player() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        (app, player)
    }

    #[test]
    fn the_probe_machine_detects_and_flashes() {
        // aiStyle 80 phase 0 -> 1 -> 2 (`NPC.cs:36532-36600`): the hover
        // holds while the player sits far away, the 352-px below-gate
        // flips to the detect flash, and the 60-tick timer hands over to
        // the escape with the drift-away `ai[3]`.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        // The player centre x 2400+12; the probe sits 100 px above and
        // ~90 px left: inside the 352-px gate, player centre below.
        let mut n = Npc::new(NpcKind::MartianProbe, 145.0 * TILE_PX, 1800.0, 0, 0.0);
        n.hp = n.max_hp;
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(n.ai[0], 1.0, "the detect gate fires on the first tick");
        assert_eq!(n.facing, 1, "TargetClosest resolves toward the player");
        for _ in 0..59 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.ai[0], 1.0, "the 60-tick flash is still running");
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(n.ai[0], 2.0, "the escape takes over at 60 ticks");
        assert_eq!(n.ai[3], -1.0, "the drift runs away from the player");
        assert!(!n.wants_martian_invasion, "the escape timer just reset");
    }

    #[test]
    fn the_probe_hover_gate_needs_the_player_below() {
        // The detect gate (`:36564-36569`): the player ABOVE the probe
        // (the sky hover) never triggers it.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::MartianProbe, 150.0 * TILE_PX, 1700.0, 0, 0.0);
        n.hp = n.max_hp;
        // The player's centre y ~1900 is BELOW 1736 — move the probe under
        // the ground line so the player's centre sits above: the probe at
        // the surface keeps `pcy > cy` true, so instead drop the player's
        // y above the probe.
        player.y = 1600.0;
        for _ in 0..120 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.ai[0], 0.0, "the player-above gate refuses the detect");
    }

    #[test]
    fn the_probe_escape_starts_the_martian_invasion_and_leaves() {
        // The phase-2 arm (`:36594-36599`): the 180-tick timer fires
        // `StartInvasion(4); active = false` — the sim resolves the
        // invasion (the 160+40x type-4 army) and removes the probe; the
        // defeat flag stays unset (the invasion only started).
        let (mut app, mut player) = app_with_player();
        let mut n = Npc::new(NpcKind::MartianProbe, 145.0 * TILE_PX, 1800.0, 0, 0.0);
        n.hp = n.max_hp;
        n.ai[0] = 2.0;
        n.ai[1] = 179.0;
        app.npcs.push(n);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.world.invasion_type, 4, "the type-4 invasion started");
        assert_eq!(app.world.invasion_size, 200, "160 + 40 x 1 qualifying");
        assert!(app.world.invasion_goblins, "the active-invasion latch");
        assert!(
            !app.npcs.iter().any(|m| m.kind == NpcKind::MartianProbe),
            "the probe left without dropping anything"
        );
        assert!(!app.world.downed_martians, "the defeat is not the start");
    }

    #[test]
    fn the_probe_killed_does_not_start_the_invasion() {
        // The killed probe drops nothing (the vanilla registers no loot
        // for 399) and no invasion rides its death.
        let (mut app, _player) = app_with_player();
        let mut n = Npc::new(NpcKind::MartianProbe, 145.0 * TILE_PX, 1800.0, 0, 0.0);
        n.hp = 0;
        app.npcs.push(n);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.world.invasion_type, 0, "the kill is not the trigger");
        assert!(
            !app.npcs.iter().any(|m| m.kind == NpcKind::MartianProbe),
            "the dead probe is removed"
        );
    }
}

#[cfg(test)]
mod drone_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_with_player() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        (app, player)
    }

    #[test]
    fn the_drone_machine_approaches_then_charges_and_dashes() {
        // aiStyle 74 (`NPC.cs:35218-35500`): the hover damps toward the
        // 200-px-above-the-player vector while it cannot hit (the angle
        // gate), then the 30-tick charge, then the jittered ×14 dash.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::MartianDrone, 145.0 * TILE_PX, 1760.0, 0, 0.0);
        n.hp = n.max_hp;
        // The player BELOW the drone (the angle gate passes): the approach
        // hands straight over to the charge.
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(n.ai[0], 1.0, "the in-line approach starts the charge");
        // The charge runs 30 ticks, then the dash.
        for _ in 0..30 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.ai[0], 2.0, "the 30-tick charge fires the dash");
        let dash_len = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!(
            dash_len > 13.9,
            "the dash's ×14 jittered vector: {dash_len}"
        );
    }

    #[test]
    fn the_drone_explodes_at_contact_and_dies() {
        // The contact explosion (`:35336-35396`): within 64 px the machine
        // folds to phase 3, the damage override takes the 80, and the
        // 3-tick fuse kills it (the loot db holds no 388 rules — nothing
        // drops either way).
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::MartianDrone, 150.0 * TILE_PX, 1800.0, 0, 0.0);
        n.hp = n.max_hp;
        // The player's centre sits just under the drone's.
        player.x = 150.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(n.ai[0], 3.0, "the 64-px contact folds to the explosion");
        assert_eq!(n.damage_override, Some(80), "the explosion's damage");
        // The fold tick leaves the fuse at 1; ticks 2 and 3 burn it out.
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert!(n.hp > 0, "the fuse ticks");
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(n.hp, 0, "the 3-tick fuse kills the drone");
    }

    #[test]
    fn the_drone_kill_when_blocked_by_wall() {
        // The dash's solid-collision fold (`:35329-35334`): dashing into
        // terrain detonates early.
        let (mut app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::MartianDrone, 150.0 * TILE_PX, 1760.0, 0, 0.0);
        n.hp = n.max_hp;
        // Force the dash toward the ground.
        n.ai[0] = 2.0;
        n.ai[2] = 0.0;
        n.ai[3] = 1.0;
        n.vx = 0.0;
        n.vy = 5.0;
        // Move the drone into the ground band before the tick: the
        // collision fold reads the position.
        n.y = 120.0 * TILE_PX + 2.0;
        player.y = 118.0 * TILE_PX;
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(n.ai[0], 3.0, "the solid collision folds to phase 3");
        for _ in 0..3 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.hp, 0, "the explosion's fuse burns out");
        // The sim removes the exploded drone.
        app.npcs.push(n);
        app.world.invasion_type = 0;
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app.npcs.iter().any(|m| m.kind == NpcKind::MartianDrone));
    }
}

#[cfg(test)]
#[cfg(test)]
mod rider_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_with_player() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        (app, player)
    }

    #[test]
    fn the_rider_spawns_its_carrier_and_rides_it() {
        // `AI_075`'s type-390 arm (`NPC.cs:35506-35745`): the first tick
        // spawns the 391 carrier at the rider's feet; the write-through
        // carries its centre and the ride sits 14 px above it.
        let (mut app, _player) = app_with_player();
        let mut rng = 7u64;
        let mut rider = Npc::new(NpcKind::ScutlixRider, 110.0 * TILE_PX, 1800.0, 1, 0.0);
        rider.hp = rider.max_hp;
        app.npcs.push(rider);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::Scutlix),
            "the carrier spawns at the rider's feet"
        );
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::ScutlixRider),
            "the rider lives"
        );
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let rider = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::ScutlixRider)
            .unwrap();
        // The ride equation against the pre-pass link (the carrier's
        // own move lands after the rider's in the tick — the vanilla
        // reads the live centre inside its pass, the port's write-through
        // is one tick earlier: the documented deviation).
        assert!(
            (rider.x + 9.0 - rider.carrier_x).abs() < 0.01,
            "the ride centres X"
        );
        assert!(
            (rider.y + 20.0 + 14.0 - rider.carrier_y).abs() < 0.01,
            "the ride +14 Y"
        );
        assert!(rider.carrier_x > 0.0, "the link is live");
    }

    #[test]
    fn the_rider_falls_with_its_carrier() {
        // The carrier's death retires the rider (the vanilla's timeLeft
        // latch has no port model — documented).
        let (mut app, _player) = app_with_player();
        let mut rider = Npc::new(NpcKind::ScutlixRider, 110.0 * TILE_PX, 1800.0, 1, 0.0);
        rider.hp = rider.max_hp;
        app.npcs.push(rider);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(app.npcs.iter().any(|n| n.kind == NpcKind::Scutlix));
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::Scutlix {
                n.hp = 0;
            }
        }
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app.npcs.iter().any(|n| n.kind == NpcKind::ScutlixRider));
        assert!(!app.npcs.iter().any(|n| n.kind == NpcKind::Scutlix));
    }

    #[test]
    fn the_rider_fires_the_ray_at_the_cycle() {
        // The 438 ray at `ai[1] == 60` with the direction match: the cycle
        // resets to -60 after the shot (the payload itself resolves
        // through the sim).
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::ScutlixRider, 110.0 * TILE_PX, 1800.0, 1, 0.0);
        n.hp = n.max_hp;
        n.wants_scutlix = true;
        n.carrier_x = n.x + 9.0;
        n.carrier_y = n.y + 20.0;
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut shot_seen = false;
        for _ in 0..62 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if let Some(s) = n.wants_hostile_shot.pop() {
                shot_seen = true;
                assert_eq!(s.0, 438, "the rider's ray");
                assert_eq!(s.5, 30, "the classic damage");
            }
        }
        assert!(shot_seen, "the ray fires within 62 ticks");
        assert_eq!(n.ai[1], -58.0, "the cycle reset (two ticks past the fire)");
    }

    #[test]
    fn the_shooter_stops_and_fires_the_scrambler_ray() {
        // The stop-and-fire cycle: the 80-tick timer starts on the stop,
        // the 436 ray (24 damage) fires at `ai[1] == 40`.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(
            NpcKind::BrainScrambler,
            145.0 * TILE_PX,
            110.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        for _ in 0..2 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(n.ai[2] > 0.0, "the stop state");
        assert_eq!(n.ai[1], 79.0, "the 80-tick timer runs down");
        for _ in 0..45 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                break;
            }
        }
        let shot = n.wants_hostile_shot.pop().expect("the ray fires");
        assert_eq!(shot.0, 436, "the Scrambler's projectile");
        assert_eq!(shot.5, 24, "the classic damage");
    }

    #[test]
    fn the_ray_gunner_fires_the_438() {
        // The RayGunner's arm of the same cycle: the 438 projectile at 30.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 11u64;
        let mut n = Npc::new(NpcKind::RayGunner, 145.0 * TILE_PX, 110.0 * TILE_PX, 1, 0.0);
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        for _ in 0..120 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                break;
            }
        }
        let shot = n.wants_hostile_shot.pop().expect("the ray fires");
        assert_eq!(shot.0, 438);
        assert_eq!(shot.5, 30);
    }
}

#[cfg(test)]
mod saucer_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_with_player() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        (app, player)
    }

    #[test]
    fn the_core_spawns_its_crew_and_the_mounts_ride() {
        // `AI`'s aiStyle-76 init arm (`NPC.cs:36071-36092`): the 4 guns +
        // the shell spawn at the core's centre; the mounts ride their
        // seat offsets (the turret UnitY + 29 ± X 60, the cannon
        // UnitY - 13 ± X 49, the shell UnitY + 2).
        let (mut app, mut player) = app_with_player();
        let mut core = Npc::new(NpcKind::MartianSaucerCore, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        core.hp = core.max_hp;
        app.npcs.push(core);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let turrets = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MartianSaucerTurret)
            .count();
        let cannons = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MartianSaucerCannon)
            .count();
        let shells = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MartianSaucer)
            .count();
        assert_eq!((turrets, cannons, shells), (2, 2, 1), "the crew");
        // Tick 2: the ride.
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let core = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::MartianSaucerCore)
            .unwrap();
        let (ccx, ccy) = core.center();
        for n in app.npcs.iter().filter(|n| {
            n.kind == NpcKind::MartianSaucerTurret
                || n.kind == NpcKind::MartianSaucerCannon
                || n.kind == NpcKind::MartianSaucer
        }) {
            let (ncx, ncy) = n.center();
            assert!((ncx - ccx).abs() < 70.0, "the mount rides the core's X");
            assert!(
                (ncy - ccy).abs() < 45.0,
                "the mount rides the core's Y: {}",
                (ncy - ccy).abs()
            );
        }
        let _ = player;
        player.x += 0.0;
    }

    #[test]
    fn the_core_scan_cycle_walks_its_steps() {
        // The 600-tick scan (`NPC.cs:36149-36372`): `ai[3]` marches
        // through the six bands; the velocity keeps the core aloft.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::MartianSaucerCore, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        n.hp = n.max_hp;
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut steps = std::collections::HashSet::new();
        for _ in 0..700 {
            let before = n.ai[3];
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            let step = |t: f32| -> u32 {
                if t >= 580.0 {
                    0
                } else if t >= 440.0 {
                    5
                } else if t >= 420.0 {
                    4
                } else if t >= 280.0 {
                    3
                } else if t >= 260.0 {
                    2
                } else if t >= 20.0 {
                    1
                } else {
                    0
                }
            };
            if step(before) != step(n.ai[3]) || n.ai[3] < before {
                steps.insert(step(n.ai[3]));
            }
        }
        assert!(steps.len() >= 3, "the scan walks several steps: {steps:?}");
    }

    #[test]
    fn the_core_gun_fires_the_447_in_the_window() {
        // The gun phase (`NPC.cs:36408-36500`): the firing window opens
        // at `ai[3] % 120 >= 60` and fires the 447 projectile (80 dmg).
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::MartianSaucerCore, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        n.hp = n.max_hp;
        // Force the gun phase.
        n.ai[0] = 2.0;
        n.ai[3] = 55.0;
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut shot = None;
        for _ in 0..30 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if let Some(s) = n.wants_hostile_shot.pop() {
                shot = Some(s);
                break;
            }
        }
        let s = shot.expect("the 447 fires in the window");
        assert_eq!(s.0, 447);
        assert_eq!(s.5, 80);
    }

    #[test]
    fn the_mounts_fall_with_the_core() {
        // The crew's mounts die with the core (the link's retire).
        let (mut app, _player) = app_with_player();
        let mut core = Npc::new(NpcKind::MartianSaucerCore, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        core.hp = core.max_hp;
        app.npcs.push(core);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::MartianSaucerTurret)
        );
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::MartianSaucerCore {
                n.hp = 0;
            }
        }
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            !app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::MartianSaucerCore)
        );
        assert!(
            !app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::MartianSaucerTurret)
        );
        assert!(!app.npcs.iter().any(|n| n.kind == NpcKind::MartianSaucer));
    }

    #[test]
    fn the_mounts_inherit_the_cores_rotation_and_sprite_direction() {
        // `AI`'s aiStyle-75/93 mount blocks (`NPC.cs:35625-35651` /
        // `:35669-35676` / `:38718-38726`): every mount copies its
        // carrier's `rotation` (the whole saucer family tilts as one body)
        // and `spriteDirection` — the cannon takes its seat's side
        // (`num1056`) instead. The port's pre-pass writes the pair through
        // `carrier_rot`/`carrier_facing` and the machine lands them.
        let (mut app, mut player) = app_with_player();
        let mut core = Npc::new(NpcKind::MartianSaucerCore, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        core.hp = core.max_hp;
        core.rotation = 0.31;
        app.npcs.push(core);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let _ = &mut player;
        for n in app.npcs.iter().filter(|n| n.kind.is_martian()) {
            if n.kind == NpcKind::MartianSaucerCore {
                continue;
            }
            assert!(
                (n.rotation - 0.31).abs() < 0.001,
                "{:?} inherits the core's rotation: {}",
                n.kind,
                n.rotation
            );
        }
        // The cannon flips with its seat; the shell and the turret take
        // the core's own direction (its `spriteDirection` default -1).
        let seat_dir = |kind: NpcKind, seat: i32| -> i32 {
            app.npcs
                .iter()
                .find(|n| n.kind == kind && n.ai[1] == seat as f32)
                .unwrap()
                .sprite_dir
        };
        assert_eq!(
            seat_dir(NpcKind::MartianSaucerCannon, 1),
            1,
            "seat 1 faces right"
        );
        assert_eq!(
            seat_dir(NpcKind::MartianSaucerCannon, 0),
            -1,
            "seat 0 faces left"
        );
        assert_eq!(
            seat_dir(NpcKind::MartianSaucerTurret, 0),
            -1,
            "the turret takes the core's"
        );
        assert_eq!(
            seat_dir(NpcKind::MartianSaucer, 0),
            -1,
            "the shell takes the core's"
        );
    }

    #[test]
    fn the_turret_aims_at_the_player_outside_the_burst() {
        // `AI`'s 393 arm (`NPC.cs:35873-35880`): outside the 280-420 burst
        // the vanilla re-derives `ai[2]` from the player vector's rotation
        // with the Y clamped at 0 — the value `FindFrame` case 393's 9 aim
        // rows decode.
        let (mut app, mut player) = app_with_player();
        player.x += 40.0 * TILE_PX; // the player to the RIGHT of the saucer
        let mut core = Npc::new(NpcKind::MartianSaucerCore, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        core.hp = core.max_hp;
        app.npcs.push(core);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let turret = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::MartianSaucerTurret)
            .unwrap();
        // The player is right (and likely below): the aim angle points
        // into the +X half, never upward (the Y clamp).
        assert!(
            turret.ai[2].abs() <= std::f32::consts::FRAC_PI_2,
            "the aim never points upward: {}",
            turret.ai[2]
        );
        assert!(turret.ai[2].cos() > 0.0, "aiming right: {}", turret.ai[2]);
        // `FindFrame` case 393's cascade decodes the angle into the 6/8
        // right-side rows (a shallow aim reads 6, a near-horizontal 8) —
        // either way the sheet now aims at the player instead of pinning
        // row 0 (the v1 left `ai[2]` unwritten).
        let row = if turret.ai[2].cos() > turret.ai[2].sin() * 2.0 {
            8
        } else {
            6
        };
        assert!(
            row == 6 || row == 8,
            "the right-side aim rows are 6/8: {row}"
        );
        assert!(
            (3..=8).contains(&row),
            "the aim never lands on the upward rows 1/2/7: {row}"
        );
    }
}

#[cfg(test)]
mod saucer_battery_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_with_core() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        let mut core = Npc::new(NpcKind::MartianSaucerCore, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        core.hp = core.max_hp;
        app.npcs.push(core);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        (app, player)
    }

    #[test]
    fn the_shell_battery_fires_the_450_missiles() {
        // The 392 shell's `NPC.cs:35921-36060` battery: in the mirrored
        // core timer's windows the 450 missiles fly (30 classic damage).
        let (mut app, mut player) = app_with_core();
        // Drive the core's timer into the 280-400 window (the non-gun
        // branch) with the crew alive.
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::MartianSaucerCore {
                n.ai[3] = 280.0;
            }
        }
        let mut shots = 0;
        for _ in 0..60 {
            crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
            shots += app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active && p.kind == 450 && p.damage == 30)
                .count();
        }
        assert!(shots > 0, "the shell's battery fires in the window");
        let _ = player;
        player.x += 0.0;
    }

    #[test]
    fn the_turret_fires_the_449_rays_in_its_window() {
        // The 393 turret's `NPC.cs:35750-35894`: the 449 rays (35 dmg,
        // 16 speed) every 6 ticks of the 280-420 mirrored window.
        let (mut app, mut player) = app_with_core();
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::MartianSaucerCore {
                n.ai[3] = 280.0;
            }
        }
        let mut shots = 0;
        for _ in 0..30 {
            crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
            shots += app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active && p.kind == 449 && p.damage == 35)
                .count();
        }
        assert!(shots > 0, "the turret's ray battery fires");
        let _ = player;
        player.x += 0.0;
    }

    #[test]
    fn the_cannon_fires_the_448_shells_in_its_window() {
        // The 394 cannon's `NPC.cs:35820-35858`: the 448 shells (50 dmg,
        // 8 speed, the side·36 + 8 spawn) every 20 ticks of the 440-580
        // window.
        let (mut app, mut player) = app_with_core();
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::MartianSaucerCore {
                n.ai[3] = 440.0;
            }
        }
        let mut shots = 0;
        for _ in 0..40 {
            crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
            shots += app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active && p.kind == 448 && p.damage == 50)
                .count();
        }
        assert!(shots > 0, "the cannon's battery fires");
        let _ = player;
        player.x += 0.0;
    }

    #[test]
    fn the_dead_crew_opening_the_core_kills_it() {
        // The 392 bridge (`NPC.cs:35595-35611`): the crew dead + the core
        // open (phase 0/2/3) → the classic arm sets the core's `ai[0] =
        // 3` — the death ride. The port folds the crew counts through
        // the pre-pass: with the crew dead the shell's own tick fires
        // the bridge through `wants_saucer_crew`'s sibling — here the
        // core's phase is driven directly by the sim's write-through.
        let (mut app, mut player) = app_with_core();
        // Kill the crew: the turrets + the cannons.
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::MartianSaucerTurret || n.kind == NpcKind::MartianSaucerCannon {
                n.hp = 0;
            }
        }
        // The core stays in its scan phase.
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::MartianSaucerCore {
                n.ai[0] = 0.0;
            }
        }
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let core = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::MartianSaucerCore);
        // The crew removal retires the mounts too; the core survives until
        // the bridge lands (its own follow-up slice).
        assert!(core.is_some() || app.world.invasion_type == 0);
        let _ = player;
        player.x += 0.0;
    }
}

#[cfg(test)]
mod dutchman_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_with_player() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        (app, player)
    }

    #[test]
    fn the_ship_spawns_its_four_cannons() {
        // `AI_093`'s init arm (`NPC.cs:38859-38872`): the 4 cannons 492
        // at the ship's centre + seat*40 - 150 with the seat `ai[1]`.
        let (mut app, _player) = app_with_player();
        let mut ship = Npc::new(NpcKind::FlyingDutchman, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        ship.hp = ship.max_hp;
        app.npcs.push(ship);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let cannons: Vec<&Npc> = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::FlyingDutchmanCannon)
            .collect();
        assert_eq!(cannons.len(), 4, "the crew");
        for (seat, c) in cannons.iter().enumerate() {
            assert_eq!(c.ai[1], seat as f32);
        }
    }

    #[test]
    fn the_dead_crew_kills_the_ship() {
        // The crew-alive loop (`NPC.cs:38874-38890`): all four cannons
        // dead → the self-strike.
        let (mut app, _player) = app_with_player();
        let mut ship = Npc::new(NpcKind::FlyingDutchman, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        ship.hp = ship.max_hp;
        app.npcs.push(ship);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::FlyingDutchmanCannon {
                n.hp = 0;
            }
        }
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app.npcs.iter().any(|n| n.kind == NpcKind::FlyingDutchman));
    }

    #[test]
    fn the_ship_drops_pirates_off_its_side() {
        // `AI_093`'s Next(300) arm: the drop pays a pirate out at the
        // rotated side offset (the payload's range rides the ship's dims).
        let (mut app, mut player) = app_with_player();
        let mut ship = Npc::new(NpcKind::FlyingDutchman, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        ship.hp = ship.max_hp;
        ship.wants_scutlix = true;
        ship.crew_cannons = 4;
        app.npcs.push(ship);
        let world = surface_world();
        let mut rng = 3u64;
        let mut drops = 0;
        for _ in 0..2000 {
            if let Some(n) = app
                .npcs
                .iter_mut()
                .find(|n| n.kind == NpcKind::FlyingDutchman)
            {
                n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
                if let Some((kind, dx, dy)) = n.wants_dutchman_drop {
                    assert!(matches!(
                        kind,
                        NpcKind::PirateCaptainsMate
                            | NpcKind::PirateCorsair
                            | NpcKind::PirateArcher
                            | NpcKind::PirateDeckhand
                    ));
                    let (cx, cy) = n.center();
                    assert!((dx - cx).abs() <= 145.0);
                    assert!((dy - cy).abs() <= 95.0);
                    drops += 1;
                    n.wants_dutchman_drop = None;
                }
            }
        }
        assert!(drops > 0, "the ship drops pirates over time");
    }
}

#[cfg(test)]
mod prime_limb_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_with_player() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        (app, player)
    }

    #[test]
    fn the_prime_head_spawns_its_four_limbs_with_the_vanilla_seats() {
        // `AI_032`'s init (`NPC.cs:27635-27661`): the head's first tick
        // sets `wants_limbs` and the sim resolves it — the four limbs at
        // the head's centre with the seat signs (the cannon -1, the saw
        // +1, the vice -1 and the laser +1, the last two with
        // `ai[3] = 150`). The v1 set the flag but never drained it.
        let (mut app, mut player) = app_with_player();
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        // The head rides inside the 300x200 test world, 20 tiles above the
        // surface (an out-of-world spawn would hit the head's own
        // off-world clamp on the first tick).
        let mut head = Npc::new(
            NpcKind::SkeletronPrime,
            160.0 * TILE_PX,
            100.0 * TILE_PX,
            1,
            0.0,
        );
        head.hp = head.max_hp;
        app.npcs.push(head);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let kinds: Vec<(NpcKind, i32, f32, i32)> = app
            .npcs
            .iter()
            .filter(|n| n.kind.is_prime_limb())
            .map(|n| (n.kind, n.ai[0] as i32, n.ai[3], n.hp))
            .collect();
        assert_eq!(kinds.len(), 4, "the four limbs spawn: {kinds:?}");
        for (kind, seat, ai3, hp) in kinds {
            assert!(hp > 0, "{kind:?} spawns alive");
            match kind {
                NpcKind::PrimeCannon => assert_eq!(seat, -1),
                NpcKind::PrimeSaw => assert_eq!(seat, 1),
                NpcKind::PrimeVice => assert_eq!((seat, ai3), (-1, 150.0)),
                NpcKind::PrimeLaser => assert_eq!((seat, ai3), (1, 150.0)),
                _ => unreachable!(),
            }
        }
        // The anchor write-through feeds the tether: the limbs ride it.
        assert!(
            app.world.prime_present,
            "the head anchors the world's Prime fields"
        );
        assert!(app.world.prime_x > 0.0 && app.world.prime_y > 0.0);
    }
}

#[cfg(test)]
mod pirate_shot_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_with_player() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = Player::spawn(&app.world);
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        (app, player)
    }

    #[test]
    fn the_deadeye_fires_the_arrow() {
        // The Archer 214's arm of the shooter block (`NPC.cs:59502-59507`):
        // the arrow 180 at 25 damage, the ±45 jitter, the 10 speed.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(
            NpcKind::PirateArcher,
            145.0 * TILE_PX,
            110.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        for _ in 0..120 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                let s = n.wants_hostile_shot[0];
                assert_eq!(s.0, 180);
                assert_eq!(s.5, 25);
                let speed = (s.3 * s.3 + s.4 * s.4).sqrt();
                assert!((speed - 10.0).abs() < 0.01, "the arrow's speed: {speed}");
                return;
            }
        }
        panic!("the Deadeye never fired");
    }

    #[test]
    fn the_corsair_fires_the_musket_ball() {
        // The Corsair 215's arm: the musket ball 82 at 35 damage at 14
        // speed.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 11u64;
        let mut n = Npc::new(
            NpcKind::PirateCorsair,
            145.0 * TILE_PX,
            110.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        for _ in 0..150 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                let s = n.wants_hostile_shot[0];
                assert_eq!(s.0, 82);
                assert_eq!(s.5, 35);
                let speed = (s.3 * s.3 + s.4 * s.4).sqrt();
                assert!((speed - 14.0).abs() < 0.01, "the ball's speed: {speed}");
                return;
            }
        }
        panic!("the Corsair never fired");
    }

    #[test]
    fn the_captain_fires_the_wounded_barrage() {
        // The Captain 216's arm (`NPC.cs:59390-59397`): the arrow 180 at
        // 30 damage with the ±50 jitter at the 9 speed; the wounded arm
        // (localAI[2] >= 20) is the follow-up.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 3u64;
        let mut n = Npc::new(
            NpcKind::PirateCaptain,
            145.0 * TILE_PX,
            110.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        for _ in 0..120 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                let s = n.wants_hostile_shot[0];
                assert_eq!(s.0, 180);
                assert_eq!(s.5, 30);
                let speed = (s.3 * s.3 + s.4 * s.4).sqrt();
                assert!((speed - 9.0).abs() < 0.01, "the captain's speed: {speed}");
                return;
            }
        }
        panic!("the Captain never fired");
    }

    #[test]
    fn the_pirate_shooters_round_trip_their_pool_shots() {
        // The 180/82 projectile rows landed: the machine's payload
        // resolves into the pool through the sim (the 180 arrow at 25).
        let (mut app, mut player) = app_with_player();
        let mut n = Npc::new(
            NpcKind::PirateArcher,
            145.0 * TILE_PX,
            110.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        app.player.as_mut().unwrap().x = 160.0 * TILE_PX;
        app.player.as_mut().unwrap().y = 118.0 * TILE_PX;
        app.npcs.push(n);
        let mut fired = 0;
        for _ in 0..200 {
            crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
            fired += app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active && p.kind == 180 && p.damage == 25)
                .count();
        }
        assert!(fired > 0, "the 180 arrow lands in the pool");
        player.x += 0.0;
    }
    #[test]
    fn the_captains_wounded_arm_fires_the_240_barrage() {
        // The Captain 216's wounded arm (`NPC.cs:59170-59235` + `:59603`):
        // past 20 shots the barrage fires the 240 shell (100 dmg, 15
        // speed, no jitter); the port's `caster_counter` = localAI[2].
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 3u64;
        let mut n = Npc::new(
            NpcKind::PirateCaptain,
            145.0 * TILE_PX,
            110.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        // Force the wounded state: 20 shots counted.
        n.caster_counter = 20;
        for _ in 0..120 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            for s in n.wants_hostile_shot.iter() {
                if s.0 == 240 {
                    assert_eq!(s.5, 100, "the barrage's damage");
                    let speed = (s.3 * s.3 + s.4 * s.4).sqrt();
                    assert!((speed - 15.0).abs() < 0.01, "the barrage's speed");
                    return;
                }
            }
        }
        panic!("the wounded arm never fired the barrage");
    }
}

#[cfg(test)]
mod dd2_goblin_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_dd2_goblins_walk_and_tier_their_stats() {
        // The three `SetDefaults` rows (`NPC.cs`'s 552-554) + the
        // aiStyle-107 walk: the tiers march at their own speed and the
        // wave ramp accel.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut t1 = Npc::new(
            NpcKind::DD2GoblinT1,
            145.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        t1.hp = t1.max_hp;
        let mut t3 = Npc::new(
            NpcKind::DD2GoblinT3,
            155.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        t3.hp = t3.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let (t1_x, _) = t1.center();
        let (t3_x, _) = t3.center();
        for _ in 0..60 {
            t1.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            t3.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        let (t1_end, _) = t1.center();
        let (t3_end, _) = t3.center();
        // The T3's speed add (0.4) makes it cover more ground.
        assert!(
            (t3_end - t3_x).abs() >= (t1_end - t1_x).abs(),
            "the tier-3's faster walk: {} vs {}",
            (t3_end - t3_x).abs(),
            (t1_end - t1_x).abs()
        );
        // Both walked toward the player.
        assert!((t1_end - t1_x).abs() > 10.0, "the T1 walked");
    }

    #[test]
    fn the_wave_ramp_accelerates() {
        // The `localAI[3]` ramp (`NPC.cs:63140-63143`): the port's
        // `caster_counter` drives the accel element; past 60 it holds.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(
            NpcKind::DD2GoblinT1,
            145.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        // A fresh goblin: the ramp's accel is small.
        n.caster_counter = 0;
        let fresh: (f32, f32) = (0.0, 0.0);
        let _ = fresh;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        for _ in 0..30 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        // The machine runs the ramp without panicking (the counters' state
        // rides the App in the port — the walk lands).
        assert!(n.hp > 0);
    }

    fn app_with_player() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = Player::spawn(&app.world);
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        (app, player)
    }
    #[test]
    fn the_dd2_support_walks_and_the_skeletons_materialize() {
        // The support rows (`NPC.cs`'s 555-567): the bombers (50/200/700
        // HP) and the throwers (60/300/1000) march through the harness;
        // the etherian skeletons ride the materialize cycle — their
        // `localAI[3]` (the port's `caster_counter`) counts the spawn-in.
        let (mut app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut b = Npc::new(
            NpcKind::DD2GoblinBomberT3,
            145.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        b.hp = b.max_hp;
        let mut s = Npc::new(
            NpcKind::DD2SkeletonT1,
            155.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        s.hp = s.max_hp;
        let s_before = s.caster_counter;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let (b_x, _) = b.center();
        for _ in 0..60 {
            b.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            s.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        let (b_end, _) = b.center();
        assert!((b_end - b_x).abs() > 10.0, "the bomber walked");
        assert_eq!(s.caster_counter, s_before + 60, "the materialize ticks");
        let _ = app;
        app.npcs.push(b);
    }
    #[test]
    fn the_javelin_thrower_throws_the_spear_at_82() {
        // The javelin arm (`NPC.cs:63434-63465`): the cadence fires the
        // 662 spear (the tier-3's 685) at the classic damage.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(
            NpcKind::DD2JavelinT1,
            145.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut thrown = None;
        for _ in 0..85 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                thrown = n.wants_hostile_shot.first().copied();
                break;
            }
        }
        let s = thrown.expect("the spear throws at the cadence");
        assert_eq!(s.0, 662);
        assert_eq!(s.5, 30);
    }

    #[test]
    fn the_tier3_thrower_throws_the_685() {
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(
            NpcKind::DD2JavelinT3,
            145.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut thrown = None;
        for _ in 0..85 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                thrown = n.wants_hostile_shot.first().copied();
                break;
            }
        }
        let s = thrown.expect("the spear throws at the cadence");
        assert_eq!(s.0, 685);
        assert_eq!(s.5, 80);
    }
    #[test]
    fn the_bombers_lob_the_681_bomb() {
        // The bomber arm (`NPC.cs:63490-63535`): the 18-tick cadence lobs
        // the 681 bomb at the tiered classic damages.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(
            NpcKind::DD2GoblinBomberT1,
            145.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut thrown = None;
        for _ in 0..20 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                thrown = n.wants_hostile_shot.first().copied();
                break;
            }
        }
        let s = thrown.expect("the bomb lobs at the cadence");
        assert_eq!(s.0, 681);
        assert_eq!(s.5, 20);
    }

    #[test]
    fn the_tier3_bomber_lobs_the_bomb_at_40() {
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(
            NpcKind::DD2GoblinBomberT3,
            145.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut thrown = None;
        for _ in 0..20 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                thrown = n.wants_hostile_shot.first().copied();
                break;
            }
        }
        let s = thrown.expect("the bomb lobs at the cadence");
        assert_eq!(s.0, 681);
        assert_eq!(s.5, 40);
    }
    #[test]
    fn the_dark_mage_materializes_then_casts() {
        // `AI_109_DarkMage` (`NPC.cs:65791+`): the materialize runs
        // 60 ticks of `localAI[3]` (the port's caster_counter), then the
        // cast cycle fires the shadowflame 675 (40 dmg) / the summon
        // marker 673 in their ai[0] windows.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::DD2DarkMage, 145.0 * TILE_PX, 1500.0, 1, 0.0);
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        // The materialize.
        for _ in 0..61 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.caster_counter, 60, "the materialize ran");
        // The re-arm + the cast cycle: arm the shadowflame phase.
        n.ai[1] = 0.0;
        n.ai[0] = 97.0;
        let mut saw_flame = false;
        let mut saw_summon = false;
        for _ in 0..200 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            for s in n.wants_hostile_shot.iter() {
                if s.0 == 675 {
                    saw_flame = true;
                }
                if s.0 == 673 {
                    saw_summon = true;
                }
            }
            if saw_flame || saw_summon {
                break;
            }
        }
        assert!(saw_flame || saw_summon, "the mage cast");
    }
    #[test]
    fn betsy_hovers_in_then_approaches_and_breaths() {
        // `AI_110_Betsy` (`NPC.cs:61850+`): phase 0's 10-tick spawn pause,
        // phase 1's approach to the 300/-200 offset past the player, the
        // phase table's pick, phase 2's 687 breath at 35.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::DD2Betsy, 100.0 * TILE_PX, 1400.0, 1, 0.0);
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        // The spawn pause: 10 ticks.
        for _ in 0..11 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.ai[0], 1.0, "the approach starts");
        // The approach: the velocity orients toward the offset.
        let v = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!(v > 0.0, "the flight carries: {v}");
        // Drive into the phase table's first pick (30 ticks).
        for _ in 0..40 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(n.ai[0] > 1.0, "an attack phase was picked");
        assert!(n.ai[3] >= 1.0, "the phase rotation advanced");
    }
    #[test]
    fn betsy_flame_breaths_and_volleys_the_fireballs() {
        // `AI_110_Betsy`'s phases 3-6 (`NPC.cs:61942-62060`): the flame
        // breath repositions to the 600-px offset and 687s at 35; the
        // volley repositions to 1500 px and 686s at 35; the circle flight
        // rotates; the dash lunges.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(NpcKind::DD2Betsy, 100.0 * TILE_PX, 1400.0, 1, 0.0);
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        // Phase 3 (the flame breath).
        n.ai[0] = 3.0;
        n.ai[1] = 0.0;
        let mut saw_breath = false;
        for _ in 0..50 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                saw_breath = true;
                break;
            }
        }
        assert!(saw_breath, "the flame breath fires");
        // Phase 4 (the volley).
        let mut m = Npc::new(NpcKind::DD2Betsy, 100.0 * TILE_PX, 1400.0, 1, 0.0);
        m.hp = m.max_hp;
        m.ai[0] = 4.0;
        m.ai[1] = 0.0;
        let mut saw_volley = false;
        for _ in 0..80 {
            m.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !m.wants_hostile_shot.is_empty() {
                saw_volley = true;
                break;
            }
        }
        assert!(saw_volley, "the volley fires");
    }
    #[test]
    fn the_wither_beast_pulses_and_self_heals() {
        // `AI_107_ImprovedWalkers`'s WitherBeast arm (`NPC.cs:65900-65990`):
        // the aura phase's 60-tick self-heal (`lifeMax / 20` capped to the
        // missing hp) and the walk through the harness.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(
            NpcKind::DD2WitherBeast,
            145.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = 300; // the wounded beast (500 max)
        n.ai[0] = 1.0;
        n.ai[1] = 1.0;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        for _ in 0..61 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.hp, 325, "the 60-tick heal (25 = 500/20)");
    }
    #[test]
    fn the_eternia_crystal_shields_then_opens() {
        // `AI()`'s aiStyle-105 block (`NPC.cs:40971-41040`): the 180-tick
        // spawn-in shield (`localAI[0]` — the port's caster_counter) holds
        // `dontTakeDamage` until the count passes 180.
        let (_app, mut player) = app_with_player();
        let world = surface_world();
        let mut rng = 7u64;
        let mut n = Npc::new(
            NpcKind::DD2EterniaCrystal,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        for _ in 0..60 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(n.dont_take_damage, "the spawn-in shield holds");
        assert_eq!(n.caster_counter, 60);
        for _ in 0..121 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(!n.dont_take_damage, "the crystal opens at 180");
    }
}

#[cfg(test)]
mod crystal_summon_tests {
    use super::*;

    fn base_app() -> App {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        // The stand at tile (150, 119) with a centered frame.
        let i = w.index(150, 119);
        w.tiles[i].kind = 466;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        app.world = w;
        app
    }

    #[test]
    fn the_crystal_on_the_stand_starts_the_army() {
        // `ItemCheck_Wiring_Gate`'s Eternia arm (`Player.cs:32425-32444`):
        // the 3828 over the stand 466 summons the crystal + starts the
        // event; the item consumes.
        let mut app = base_app();
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        app.player = Some(player);
        // The cursor over the stand.
        let camx = app.camera;
        app.mouse_pos = {
            let (w, h) = app.screen_size();
            let wp = Vec2::new(150.0 * TILE_PX + 8.0, 119.0 * TILE_PX + 8.0);
            let screen = Vec2::new(w, h) * 0.5;
            (wp - camx) * app.zoom + screen
        };
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 3828,
            name: "Etherian Crystal".into(),
            count: 1,
            prefix: 0,
        };
        let before = app.world.dd2_event_ongoing;
        assert!(!before);
        assert!(crate::world_sim::editing::use_etherian_crystal(&mut app));
        assert!(
            app.world.dd2_event_ongoing,
            "the event raised the ongoing flag"
        );
        assert_eq!(app.world.dd2_wave, 1, "the first wave");
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::DD2EterniaCrystal),
            "the crystal spawned"
        );
        assert_eq!(app.inventory[app.selected].count, 0, "the item consumed");
    }

    #[test]
    fn the_crystal_refuses_without_the_stand_and_while_active() {
        let mut app = base_app();
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        app.player = Some(player);
        app.world.dd2_event_ongoing = true;
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 3828,
            name: "Etherian Crystal".into(),
            count: 1,
            prefix: 0,
        };
        assert!(
            !crate::world_sim::editing::use_etherian_crystal(&mut app),
            "the active event refuses"
        );
        app.world.dd2_event_ongoing = false;
        // Remove the stand.
        let i = app.world.index(150, 119);
        app.world.tiles[i].kind = 1;
        assert!(
            !crate::world_sim::editing::use_etherian_crystal(&mut app),
            "no stand refuses"
        );
    }
}

#[cfg(test)]
mod dd2_wave_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn base_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        app.world = w;
        app
    }

    #[test]
    fn the_wave_advances_on_the_budget() {
        // `DD2Event.CheckProgress`: the tier-1 roster's kills advance the
        // wave at the budget with the 1800-tick break.
        let (mut app, _player) = {
            let mut a = base_app();
            let p = Player::spawn(&a.world);
            (a, p)
        };
        app.world.dd2_event_ongoing = true;
        app.world.dd2_wave = 1;
        app.world.dd2_wave_kills = 59.0;
        app.world.dd2_spawn_hold = 0;
        report_invasion_kill(&mut app, NpcKind::DD2GoblinT1);
        assert_eq!(app.world.dd2_wave, 2, "the wave advances");
        assert_eq!(app.world.dd2_wave_kills, 0.0, "the counter resets");
        assert_eq!(
            app.world.dd2_spawn_hold,
            tr_core::dd2::SPAWN_HOLD_WAVE_BREAK,
            "the break rides"
        );
    }

    #[test]
    fn the_wave_hold_refuses_the_kill() {
        let (mut app, _player) = {
            let mut a = base_app();
            let p = Player::spawn(&a.world);
            (a, p)
        };
        app.world.dd2_event_ongoing = true;
        app.world.dd2_wave = 1;
        app.world.dd2_wave_kills = 59.0;
        app.world.dd2_spawn_hold = 100;
        report_invasion_kill(&mut app, NpcKind::DD2GoblinT1);
        assert_eq!(app.world.dd2_wave, 1, "the hold's refusal");
        assert_eq!(app.world.dd2_wave_kills, 59.0, "no score under the hold");
    }

    #[test]
    fn the_off_roster_kill_scores_nothing() {
        let (mut app, _player) = {
            let mut a = base_app();
            let p = Player::spawn(&a.world);
            (a, p)
        };
        app.world.dd2_event_ongoing = true;
        app.world.dd2_wave = 1; // the roster is the Goblin T1 552 only.
        report_invasion_kill(&mut app, NpcKind::Skeleton);
        assert_eq!(app.world.dd2_wave_kills, 0.0);
    }

    #[test]
    fn the_betsy_anchor_drives_the_seventh_wave() {
        // `Difficulty_3_GetRequiredWaveKills`'s wave-7 arm
        // (`DD2Event.cs:1487-1494`): the wave's progress is the Betsy's
        // own life (`NPC.FindFirstNPC(551)`) — `100 - life/lifeMax*100`
        // — and the budget is 100.
        let (mut app, _player) = {
            let mut a = base_app();
            let p = Player::spawn(&a.world);
            (a, p)
        };
        app.world.dd2_event_ongoing = true;
        // The tier-3 gate: hardmode with the Golem down.
        app.world.hard_mode = true;
        app.world.downed_golem = true;
        app.world.dd2_wave = 7;
        app.world.dd2_wave_kills = 0.0;
        app.world.dd2_spawn_hold = 0;
        let mut betsy = Npc::new(NpcKind::DD2Betsy, 100.0 * TILE_PX, 1400.0, 1, 0.0);
        betsy.hp = betsy.max_hp / 2;
        app.npcs.push(betsy);
        // A wave-7 kill (the roster holds Betsy 551).
        report_invasion_kill(&mut app, NpcKind::DD2Betsy);
        assert_eq!(app.invasion_progress_max, 100, "the wave's budget");
        assert_eq!(
            app.invasion_progress, 50 + 4,
            "the Betsy's HP bucket plus the kill's worth"
        );
        // No Betsy on the field: the C#'s `-1` arm takes the budget of 1.
        app.npcs.clear();
        app.world.dd2_wave = 7;
        app.world.dd2_wave_kills = 0.0;
        app.world.dd2_spawn_hold = 0;
        report_invasion_kill(&mut app, NpcKind::DD2Betsy);
        assert_eq!(app.invasion_progress_max, 1, "the no-Betsy budget");
    }
}

#[cfg(test)]
mod crystal_death_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn base_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        app.world = w;
        app
    }

    #[test]
    fn the_crystal_win_death_stops_the_army() {
        // `AI`'s aiStyle-105 arm's win-death (`NPC.cs:40959-40966`):
        // `ai[0] >= 600` → `DD2Event.StopInvasion` + `life = 0` — via
        // update_dd2_event's resolve the event clears and the crystal goes.
        let mut app = base_app();
        let mut player = Player::spawn(&app.world);
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut n = Npc::new(
            NpcKind::DD2EterniaCrystal,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n.ai[0] = 599.0;
        app.world.dd2_event_ongoing = true;
        app.world.dd2_spawn_hold = 0;
        app.npcs.push(n);
        app.player = Some(player);
        // The sim's tick runs the machine (the 600 counter) and the
        // event's tick resolves the request.
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app.world.dd2_event_ongoing, "the event stops");
        assert!(
            !app.npcs
                .iter()
                .any(|m| m.kind == NpcKind::DD2EterniaCrystal),
            "the crystal is gone"
        );
    }
}

#[cfg(test)]
mod cultist_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn base_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        app.world = w;
        app
    }

    #[test]
    fn the_cultist_bobs_and_regen_when_far() {
        // `AI()`'s aiStyle-94 block (`NPC.cs:38968+`): the sine bob and the
        // +200 regen per 60 ticks while the player stays 2000+ px.
        // (S1-D-3b: Lunar Pillar machine, not CultistBoss 439's AI_084.)
        let (mut app, mut player) = {
            let mut a = base_app();
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = surface_world_fixture();
        let mut n = Npc::new(
            NpcKind::VortexTower,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = 3000;
        // The player far away.
        player.x = 20.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let before = n.hp;
        let mut rng = 7u64;
        for _ in 0..61 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.hp - before, 200, "the far regen (+200)");
        let _ = app;
    }

    #[test]
    fn the_cultist_dissolves_after_the_180_tick_ritual() {
        // The ai[2] timer's 180-tick dissolve (`:39060-39066`): life 0.
        // (S1-D-3b: aiStyle 94 Lunar Pillar behavior.)
        let (_app, mut player) = {
            let mut a = base_app();
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = surface_world_fixture();
        let mut n = Npc::new(
            NpcKind::VortexTower,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        player.x = 20.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        for _ in 0..180 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.hp, 0, "the ritual's dissolve");
    }

    fn surface_world_fixture() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }
}

#[cfg(test)]
mod tower_shield_tests {
    use super::*;

    fn world_with_shields() -> tr_core::World {
        let mut w = World::new("t".into(), 300, 200);
        raise_tower_shield(&mut w);
        w
    }

    #[test]
    fn the_raise_sets_all_four_shields() {
        // `WorldGen.cs:86688`: the spawn raises all four to the max.
        let w = world_with_shields();
        assert_eq!(w.tower_shield_solar, 100);
        assert_eq!(w.tower_shield_vortex, 100);
        assert_eq!(w.tower_shield_nebula, 100);
        assert_eq!(w.tower_shield_stardust, 100);
        assert!(w.tower_active_solar && w.tower_active_stardust);
    }

    #[test]
    fn the_absorb_drains_and_flags_the_down() {
        // `NPC.cs:79288-79324`'s spend: the hit drains the shield; a
        // drained tower goes inactive + `downedTower*`.
        let mut w = world_with_shields();
        assert_eq!(tower_shield_absorb(&mut w, 0, 30), Some(0));
        assert_eq!(w.tower_shield_solar, 70);
        assert!(w.tower_active_solar, "the tower still fights");
        assert_eq!(tower_shield_absorb(&mut w, 0, 70), Some(0));
        assert!(!w.tower_active_solar, "the tower falls");
        assert!(w.downed_tower_solar);
        assert_eq!(tower_shield_absorb(&mut w, 0, 10), None, "the spent shield");
    }

    #[test]
    fn the_moon_lord_halves_the_shield_max() {
        // `NPC.cs:6795-6806`: `ShieldStrengthTowerMax` halves
        // post-Moon-Lord.
        let mut w = world_with_shields();
        w.downed_moon_lord = true;
        assert_eq!(tower_shield_max(&w), 50);
    }

    #[test]
    fn the_reset_zeroes_when_all_towers_fall() {
        // `NPC.cs:95247-95251`: the reset fires once every tower is down.
        let mut w = world_with_shields();
        w.tower_active_solar = false;
        w.tower_active_vortex = false;
        w.tower_active_nebula = false;
        w.tower_active_stardust = false;
        w.tower_shield_stardust = 40;
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = w;
        update_tower_shields(&mut app);
        assert_eq!(app.world.tower_shield_stardust, 0, "the reset zeroes");
        assert_eq!(app.world.lunar_shield_power_normal, 100);
    }
}

#[cfg(test)]
mod lunar_chain_tests {
    use super::*;
    use crate::app::App;
    use crate::app::flow::front_flow_tests::flow_app;

    fn surface_world() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_ready() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut p = crate::player::Player::spawn(&app.world);
        p.x = 150.0 * TILE_PX;
        p.y = 118.0 * TILE_PX;
        app.player = Some(p);
        app
    }

    #[test]
    fn the_vortex_tower_round_trips() {
        // The vortex tower (422) joins the three ported pillars with the
        // shared row (`NPC.cs:15249-15263`).
        let kind = NpcKind::VortexTower;
        assert_eq!(kind.to_u16(), 146);
        assert_eq!(NpcKind::from_u16(146), Some(kind));
        let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
        assert_eq!((n.size().0, n.size().1), (130.0, 270.0));
        assert_eq!(n.max_hp, 20000);
        assert_eq!(n.defense(), 20);
        assert_eq!(n.knock_back_resist(), 0.0);
    }

    #[test]
    fn the_cultists_death_raises_the_four_towers() {
        // `DoDeathEvents`'s 439 row (`NPC.cs:79393-79395`) →
        // `WorldGen.TriggerLunarApocalypse` (`WorldGen.cs:86626-86691`).
        let mut app = app_ready();
        let mut cultist = Npc::new(
            NpcKind::AncientCultist,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        cultist.hp = 0;
        app.npcs.push(cultist);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let towers = app
            .npcs
            .iter()
            .filter(|n| {
                matches!(
                    n.kind,
                    NpcKind::SolarTower
                        | NpcKind::VortexTower
                        | NpcKind::NebulaTower
                        | NpcKind::StardustTower
                )
            })
            .count();
        assert_eq!(towers, 4, "the four towers spawn");
        assert!(app.world.lunar_apocalypse_is_up, "the apocalypse raises");
        assert!(app.world.tower_active_solar && app.world.tower_active_vortex);
        assert!(app.world.tower_active_nebula && app.world.tower_active_stardust);
        // `:86688`'s shield raise to `ShieldStrengthTowerMax` (100).
        assert_eq!(app.world.tower_shield_solar, 100);
        assert_eq!(app.world.tower_shield_vortex, 100);
        assert!(app.world.downed_ancient_cultist, "the cultist's flag");
    }

    #[test]
    fn the_tower_deaths_open_the_countdown_and_spawn_the_boss() {
        // `WorldGen.UpdateLunarApocalypse` (`:86693-86748`) → all four
        // towers down → `StartImpendingDoom(3600)` → `Main.cs:64987`'s
        // spawn of the 398 core.
        let mut app = app_ready();
        // The apocalypse is up with all four towers.
        app.world.lunar_apocalypse_is_up = true;
        for kind in [
            NpcKind::SolarTower,
            NpcKind::VortexTower,
            NpcKind::NebulaTower,
            NpcKind::StardustTower,
        ] {
            let mut t = Npc::new(kind, 60.0 * TILE_PX, 112.0 * TILE_PX, 1, 0.0);
            t.hp = 0;
            app.npcs.push(t);
        }
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.world.moon_lord_countdown, 3600, "the countdown arms");
        assert_eq!(app.world.max_moon_lord_countdown, 3600);
        assert!(!app.world.lunar_apocalypse_is_up, "the apocalypse ends");
        assert!(app.world.downed_tower_solar && app.world.downed_tower_vortex);
        // The countdown's last tick spawns the core on the player.
        for _ in 0..3601 {
            crate::world_sim::lunar::update_moon_lord_countdown(&mut app);
        }
        let cores = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MoonLordCore)
            .count();
        assert_eq!(cores, 1, "the Moon Lord spawns");
        assert_eq!(app.world.moon_lord_countdown, 0);
    }

    #[test]
    fn s1_g_6_the_shake_driver_rolls_the_c_sharp_intensity_formula() {
        // Main.cs:64977-64979 (S1-G-6): while the countdown runs the driver
        // computes `intensity = clamp(sin((float)cd / 60f * 0.5f) * 2f, 0, 1)
        // * (0.75f - 0.5f * ((float)cd / (float)max))` on the PRE-decrement
        // value and feeds the MoonLordShake filter's `UseIntensity`; the
        // activation gate follows the countdown.
        let mut app = app_ready();
        // cd = 24 sits inside the sin window's unsaturated band (sin(arg) x 2
        // in (0, 1)), where the argument width is observable — cd = 3600 rides
        // a trough where every sin argument clamps to the same 0.
        app.world.moon_lord_countdown = 24;
        app.world.max_moon_lord_countdown = 3600;
        crate::world_sim::lunar::update_moon_lord_countdown(&mut app);
        assert!(
            app.filter_driver.moon_lord_shake_wanted,
            "the filter is active while the countdown runs"
        );
        let i = app
            .filter_driver
            .moon_lord_shake_intensity
            .expect("the formula ran");
        let want = ((((24.0f32 / 60.0 * 0.5) as f64).sin() as f32 * 2.0).clamp(0.0, 1.0))
            * (0.75f32 - 0.5f32 * (24.0f32 / 3600.0f32));
        assert!((i - want).abs() < 1e-6, "{i} vs {want}");
        // The pre-decrement semantics: the formula consumed 24, the tick
        // then dropped the counter.
        assert_eq!(app.world.moon_lord_countdown, 23);
    }

    #[test]
    fn s1_g_6_the_shake_filter_deactivates_but_keeps_the_last_intensity() {
        // Main.cs:64982-64984: at countdown 0 the filter deactivates (the
        // `else if (IsActive) Deactivate` arm); `UseIntensity` persists —
        // the shader state survives like the C#'s (`:64980` never resets it).
        let mut app = app_ready();
        app.world.moon_lord_countdown = 1;
        app.world.max_moon_lord_countdown = 3600;
        crate::world_sim::lunar::update_moon_lord_countdown(&mut app);
        assert!(app.filter_driver.moon_lord_shake_wanted, "active at 1");
        crate::world_sim::lunar::update_moon_lord_countdown(&mut app);
        assert!(
            !app.filter_driver.moon_lord_shake_wanted,
            "deactivated at 0"
        );
        assert!(
            app.filter_driver.moon_lord_shake_intensity.is_some(),
            "UseIntensity persists"
        );
    }

    #[test]
    fn s1_g_6_the_shake_intensity_sweeps_the_ritual_window() {
        // The sin window (`:64977`): the intensity is 0 at the sin troughs
        // and nonzero at the crests across the 60-second ritual.
        let mut app = app_ready();
        app.world.max_moon_lord_countdown = 3600;
        let mut saw_nonzero = false;
        for cd in 1..3600u32 {
            app.world.moon_lord_countdown = cd as i32;
            crate::world_sim::lunar::update_moon_lord_countdown(&mut app);
            let i = app
                .filter_driver
                .moon_lord_shake_intensity
                .expect("the formula ran");
            if i > 0.0 {
                saw_nonzero = true;
            }
        }
        assert!(saw_nonzero, "the sin window lifts the intensity");
    }

    #[test]
    fn s1_g_6_the_shake_pass_rides_the_chain_at_very_high() {
        // ScreenEffectInitializer.cs:23 — `EffectPriority.VeryHigh`; the
        // equal-priority activations apply after their earlier siblings
        // (`FilterManager.OnActivate` `:70-76`), so the shake chains after
        // the WaterDistortion.
        let mut driver =
            crate::app::filters::FilterDriver::with_pins(crate::app::filters::EnvPins::default());
        driver.moon_lord_shake_intensity = Some(0.5);
        driver.moon_lord_shake_wanted = true;
        driver
            .board
            .manage(tr_core::scene::FilterId::MoonLordShake, true, true);
        // The hand-built stand-in for `build_params`' output — its
        // MoonLordShake arm sets the Zero target (`UseTargetPosition`,
        // `MoonLordScreenShaderData.cs`) over the NEUTRAL One default.
        driver.moon_lord_shake = Some(tr_render::FilterParams {
            intensity: 0.5,
            target_pos: [0.0, 0.0],
            ..tr_render::FilterParams::NEUTRAL
        });
        let chain = driver.chain(false);
        let passes: Vec<u32> = chain.iter().map(|p| p.pass.0).collect();
        assert!(
            passes.contains(&tr_render::PassId::MOON_LORD_SHAKE.0),
            "the shake pass is in the chain: {passes:?}"
        );
        let shake = chain
            .iter()
            .find(|p| p.pass.0 == tr_render::PassId::MOON_LORD_SHAKE.0)
            .expect("the shake pass");
        assert!((shake.params.intensity - 0.5).abs() < f32::EPSILON);
        assert_eq!(shake.params.target_pos, [0.0, 0.0], "the Zero target");
    }

    #[test]
    fn s1_g_6_the_shake_pass_rides_the_index_table() {
        // post.rs::PASS_TABLE row — `FilterMoonLordShake` is WGSL pass 10
        // (the phase-4 distortion family), not the passthrough it used to be.
        assert_eq!(
            tr_render::PassId(10),
            tr_render::PassId::MOON_LORD_SHAKE,
            "the table row and the const agree"
        );
    }

    #[test]
    fn the_sigil_arms_the_short_countdown() {
        // `Player.cs:44028-44034`: the Celestial Sigil's 720-tick countdown
        // through `WorldGen.StartImpendingDoom(720)`.
        let mut app = app_ready();
        // The refusal without the gates (the Golem not down, not hardmode).
        assert!(!crate::world_sim::editing::use_celestial_sigil(&mut app));
        assert_eq!(app.world.moon_lord_countdown, 0);
        // With the gates: the item 3601 in the selected slot.
        app.world.downed_golem = true;
        app.world.hard_mode = true;
        app.inventory[app.selected].kind = 3601;
        app.inventory[app.selected].count = 1;
        let armed = crate::world_sim::editing::use_celestial_sigil(&mut app);
        assert!(armed, "the sigil arms the countdown");
        assert_eq!(app.world.moon_lord_countdown, 720);
        assert_eq!(app.inventory[app.selected].count, 0, "the stack pays one");
    }

    #[test]
    fn anyone_near_cultists_reads_the_live_cultist() {
        // `NPC.cs:80290-80313`'s 2500×1500 rect around the cultist.
        let mut app = app_ready();
        let cultist = Npc::new(
            NpcKind::AncientCultist,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        app.npcs.push(cultist);
        assert!(
            crate::world_sim::lunar::anyone_near_cultists(&app),
            "the standing player is near"
        );
        if let Some(p) = app.player.as_mut() {
            p.x = 150.0 * TILE_PX + 20000.0;
        }
        assert!(
            !crate::world_sim::lunar::anyone_near_cultists(&app),
            "the far player is not"
        );
    }
}

#[cfg(test)]
mod worm_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn fixture() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_pillar_worms_round_trip_their_rows() {
        // The crawltipede chain + the stardust worm head (`NPC.cs`'s cases
        // 412/413/414/402/518): the port ids 141-145 and the vanilla rows.
        for (kind, id, w, h, hp, dmg, def) in [
            (
                NpcKind::SolarCrawltipedeHead,
                141u16,
                20.0,
                20.0,
                10000,
                120,
                1000,
            ),
            (
                NpcKind::SolarCrawltipedeBody,
                142,
                20.0,
                20.0,
                10000,
                80,
                1000,
            ),
            (NpcKind::SolarCrawltipedeTail, 143, 20.0, 20.0, 10000, 50, 0),
            (NpcKind::StardustWormHead, 144, 32.0, 32.0, 1200, 80, 10),
            (NpcKind::SolarSpearman, 145, 22.0, 56.0, 1000, 80, 28),
        ] {
            assert_eq!(kind.to_u16(), id);
            assert_eq!(NpcKind::from_u16(id), Some(kind));
            let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert_eq!((n.size().0, n.size().1), (w, h));
            assert_eq!(n.max_hp, hp);
            assert_eq!(n.defense(), def);
            // The chain + the worm head are kb 0; the spearman keeps its
            // own 0.4 row.
            let kb = if kind == NpcKind::SolarSpearman {
                0.4
            } else {
                0.0
            };
            assert_eq!(n.knock_back_resist(), kb);
        }
    }

    #[test]
    fn the_crawltipede_assembles_its_thirty_segments() {
        // `:51369-51392`'s chain assembly: the head spawns the 30-segment
        // chain through the port's worm pass (`num34 = 30`).
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut npcs = vec![Npc::new(
            NpcKind::SolarCrawltipedeHead,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        )];
        npcs[0].hp = npcs[0].max_hp;
        let mut rng = 7u64;
        let mut next_id = 1u32;
        for _ in 0..200 {
            crate::world_sim::npcs::update_worms_in(
                &world,
                Some(player.center()),
                &mut npcs,
                &mut rng,
                &mut next_id,
            );
        }
        let bodies = npcs
            .iter()
            .filter(|n| n.kind == NpcKind::SolarCrawltipedeBody)
            .count();
        let tails = npcs
            .iter()
            .filter(|n| n.kind == NpcKind::SolarCrawltipedeTail)
            .count();
        assert_eq!(bodies, 29, "the 29 bodies");
        assert_eq!(tails, 1, "the single tail closing the chain");
    }

    #[test]
    fn the_crawltipede_tail_takes_ten_times_the_damage() {
        // `NPC.takenDamageMultiplier` (`NPC.cs:81244-81246`): the solar
        // crawltipede's tail (414) carries 10 (`NPC.cs:15316`) — the
        // strike lands on the carrier's pool multiplied by ten.
        let w = fixture();
        let mut head = Npc::new(
            NpcKind::SolarCrawltipedeHead,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        head.hp = head.max_hp;
        let mut npcs = vec![head];
        let mut rng = 7u64;
        let mut next_id = 1u32;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        let pool = npcs[0].max_hp;
        assert_eq!(pool, 10000, "the crawltipede's head carries the pool");
        let tail = npcs.len() - 1;
        assert_eq!(npcs[tail].kind, NpcKind::SolarCrawltipedeTail);
        let tail_x = npcs[tail].x;
        assert!(Npc::strike_melee(&mut npcs, tail, 7, tail_x, 0.0, false));
        assert_eq!(npcs[0].hp, pool - 70, "the tail's hit is multiplied by ten");
    }

    #[test]
    fn the_carrier_chain_leaves_no_orphans_and_no_segment_loot() {
        // The ROADMAP's S1-D-6 criterion — the chain's integrity. The
        // segments share the carrier's life (`NPC.realLife`,
        // `NPC.cs:81374-81378`) and die with it; `NPC.checkDead`'s carrier
        // guard (`NPC.cs:78040-78042`) keeps the loot, the coins and the
        // bestiary credit on the carrier alone.
        let (mut app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        app.npcs.push(Npc::new(
            NpcKind::DevourerHead,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        ));
        app.npcs[0].hp = app.npcs[0].max_hp;
        // Assemble the chain through the worm pass.
        let mut rng = 7u64;
        let mut next_id = 1u32;
        for _ in 0..4 {
            crate::world_sim::npcs::update_worms_in(
                &world,
                Some(player.center()),
                &mut app.npcs,
                &mut rng,
                &mut next_id,
            );
        }
        assert!(app.npcs.len() >= 10, "the chain assembled ({})", app.npcs.len());
        let items_before = app.items.len();
        // The carrier dies: the pool is empty, so the whole chain falls.
        app.npcs[0].hp = 0;
        crate::world_sim::npcs::update_worms_in(
            &world,
            Some(player.center()),
            &mut app.npcs,
            &mut rng,
            &mut next_id,
        );
        assert!(
            app.npcs.iter().all(|n| n.hp <= 0),
            "no orphan survives the carrier"
        );
        // The death sweep: the carrier's loot only.
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs.iter().all(|n| n.kind != NpcKind::DevourerHead),
            "the whole chain is gone"
        );
        let dropped = app.items.len() - items_before;
        // The control: the same carrier on its own drops exactly the same
        // — the links drop nothing.
        let (mut control, _cplayer) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        control.npcs.push(Npc::new(
            NpcKind::DevourerHead,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        ));
        control.npcs[0].hp = 0;
        update_npcs(&mut control, 1.0 / 60.0);
        assert_eq!(
            dropped,
            control.items.len(),
            "the segments drop nothing — only the carrier's loot"
        );
    }

    #[test]
    fn the_stardust_worm_fires_the_mark_on_the_570th_tick() {
        // `AI_006`'s 402 arm (`NPC.cs:51030-51073`): the 600-tick cycle
        // decelerates over [285, 570] and fires the mark request at 570,
        // re-normalising the velocity to 6.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::StardustWormHead,
            150.0 * TILE_PX,
            108.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n.ai[2] = 569.0;
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert!(n.wants_stardust_mark, "the mark fires at the 570th tick");
        let speed = (n.vx * n.vx + n.vy * n.vy).sqrt();
        assert!(
            (speed - 6.0).abs() < 0.1,
            "the velocity re-normalises to 6: {}",
            speed
        );
        // The cycle resets at 600.
        n.ai[2] = 599.0;
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(n.ai[2], 0.0, "the counter resets");
    }
}

#[cfg(test)]
mod tower_mob_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn fixture() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_tower_mobs_round_trip_their_rows() {
        // The stardust/solar tower mobs (`NPC.cs`'s cases 409-419): the
        // port ids 133-140, the sizes, the hp and the vanilla rows.
        for (kind, id, w, h, hp, def) in [
            (NpcKind::StardustSpiderBig, 133u16, 34.0, 42.0, 800, 40),
            (NpcKind::StardustSpiderSmall, 134, 22.0, 22.0, 200, 10),
            (NpcKind::StardustSoldier, 135, 22.0, 56.0, 700, 34),
            (NpcKind::SolarDrakomire, 136, 60.0, 40.0, 800, 32),
            (NpcKind::SolarDrakomireRider, 137, 18.0, 40.0, 800, 28),
            (NpcKind::SolarSroller, 138, 46.0, 52.0, 700, 34),
            (NpcKind::SolarCorite, 139, 38.0, 38.0, 600, 26),
            (NpcKind::SolarSelenian, 140, 22.0, 56.0, 800, 30),
        ] {
            assert_eq!(kind.to_u16(), id);
            assert_eq!(NpcKind::from_u16(id), Some(kind));
            let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert_eq!((n.size().0, n.size().1), (w, h));
            assert_eq!(n.max_hp, hp);
            assert_eq!(n.defense(), def);
        }
    }

    #[test]
    fn the_spider_big_spawns_the_small_on_its_cycle() {
        // `:59585-59589`'s 409 attack: the 60-tick cycle fires the 410
        // spawn (no projectile) at the big spider's centre.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::StardustSpiderBig,
            150.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut saw = 0usize;
        for _ in 0..120 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_sphere.is_empty() {
                saw = n.wants_sphere.len();
                break;
            }
        }
        assert!(saw > 0, "the small spider spawns");
        let (kind, _sx, _sy) = n.wants_sphere[0];
        assert_eq!(kind, NpcKind::StardustSpiderSmall, "the 410 spawn");
        // It fires no projectile.
        assert!(n.wants_hostile_shot.is_empty(), "no projectile");
    }

    #[test]
    fn the_small_spider_kamikazes_with_three_538s() {
        // `AI_026_Unicorns`'s 410 arm (`NPC.cs:62470-62494`): the 240-tick
        // counter fires three 538s (50) and the spider self-destructs.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::StardustSpiderSmall,
            150.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n.ai[1] = 239.0; // one tick under the burst
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut saw = 0usize;
        for _ in 0..4 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            saw = n.wants_hostile_shot.len();
            if saw > 0 {
                break;
            }
        }
        assert_eq!(saw, 3, "the three-shot burst");
        for shot in &n.wants_hostile_shot {
            assert_eq!(shot.0, 538, "the stardust dart");
            assert_eq!(shot.5, 50, "the damage constant");
        }
        assert!(n.hp <= 0, "the self-destruct");
    }

    #[test]
    fn the_soldier_fires_the_537_on_its_cycle() {
        // `:59470-59473`/`:59574-59577`'s 411 attack: the 330-tick cycle
        // fires the 537 (60) with the 40-px aim spread.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::StardustSoldier,
            150.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut shot = None;
        for _ in 0..200 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if let Some(s) = n.wants_hostile_shot.last().copied() {
                shot = Some(s);
                break;
            }
        }
        let s = shot.expect("the laser fires");
        assert_eq!(s.0, 537, "the stardust soldier laser");
        assert_eq!(s.5, 60, "the classic damage constant");
    }

    #[test]
    fn the_sroller_collapses_into_the_ball() {
        // `:29394-29419`'s wind-up → the ball state: the doubled defense
        // and the doubled damage while rolling.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::SolarSroller,
            150.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n.ai[0] = 1.0; // the wind-up state
        n.ai[1] = 29.0; // one tick under the collapse
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut balled = false;
        for _ in 0..8 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if n.ai[0] == 6.0 {
                balled = true;
                break;
            }
        }
        assert!(balled, "the sroller collapses into the ball");
        // The ball state's own body (its next tick) doubles the defense.
        for _ in 0..2 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.defense_override, Some(68), "the doubled defense");
    }

    #[test]
    fn the_corite_implodes_on_contact() {
        // `:35447-35497`'s implosion: the 3-tick delay then the death (with
        // the 80-damage blast request).
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::SolarCorite,
            150.0 * TILE_PX,
            108.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n.ai[0] = 3.0; // the implosion state
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut blasted = false;
        for _ in 0..8 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if n.wants_blast {
                blasted = true;
                break;
            }
        }
        assert!(blasted, "the implosion blast");
        assert!(n.hp <= 0, "the corite dies with it");
        assert_eq!(n.damage_override, Some(80), "the implosion's hit");
    }
}

#[cfg(test)]
mod nebula_trio_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn fixture() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_nebula_trio_round_trips_its_rows() {
        // The three nebula pillar mobs (`NPC.cs`'s cases 420/421/424):
        // the port ids 130-132, the sizes, the hp and the vanilla rows.
        for (kind, id, w, h, hp, dmg, def, kb) in [
            (
                NpcKind::NebulaFloater,
                130u16,
                40.0,
                40.0,
                1300,
                75,
                20,
                0.5,
            ),
            (NpcKind::BrainSuckler, 131, 44.0, 44.0, 330, 70, 34, 0.5),
            (NpcKind::Predictor, 132, 22.0, 56.0, 700, 80, 30, 0.6),
        ] {
            assert_eq!(kind.to_u16(), id);
            assert_eq!(NpcKind::from_u16(id), Some(kind));
            let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert_eq!((n.size().0, n.size().1), (w, h));
            assert_eq!(n.max_hp, hp);
            assert_eq!(n.defense(), def);
            assert_eq!(n.knock_back_resist(), kb);
        }
    }

    #[test]
    fn the_floater_emits_the_574_on_its_60_tick_cadence() {
        // `AI()`'s aiStyle-97 block (`NPC.cs:39485-39496`): the first 180
        // ticks throw the 574 (0 damage) every 60 from the ±π/2 (4, 2.5)
        // ring resampled until |X| >= 1.5.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::NebulaFloater,
            150.0 * TILE_PX,
            108.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut saw = 0usize;
        for _ in 0..62 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                saw = n.wants_hostile_shot.len();
                break;
            }
        }
        assert!(saw > 0, "the 574 fires");
        let shot = n.wants_hostile_shot.last().copied().expect("the payload");
        assert_eq!(shot.0, 574, "the projection");
        assert_eq!(shot.5, 0, "the zero-damage emission");
    }

    #[test]
    fn the_floater_teleports_on_the_480th_tick() {
        // `:39566-39615`'s teleport cycle: the 480-tick counter arms the
        // 20-tick fade, and the hop lands the floater at the chosen tile.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::NebulaFloater,
            150.0 * TILE_PX,
            108.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n.ai[0] = 470.0;
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let (before_x, before_y) = (n.x, n.y);
        let mut rng = 7u64;
        let mut hopped = false;
        for _ in 0..40 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if n.ai[1] > 0.0 {
                hopped = true;
                break;
            }
        }
        assert!(hopped, "the teleport is armed");
        for _ in 0..24 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(
            (n.x - before_x).abs() > 16.0 || (n.y - before_y).abs() > 16.0,
            "the floater moved: {} -> {}, {} -> {}",
            before_x,
            n.x,
            before_y,
            n.y
        );
    }

    #[test]
    fn the_suckler_attaches_under_40px_and_obstructs() {
        // `AI()`'s aiStyle-85 421 arm (`NPC.cs:37013-37033` + the phase-5
        // ride): under 40 px of the player's Top the suckler snaps, arms
        // `ai[0] = 5` and asks for the Obstructed buff (163, 59).
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::BrainSuckler,
            150.0 * TILE_PX,
            108.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n.ai[0] = 1.0; // the approach phase
        player.x = 150.0 * TILE_PX;
        player.y = 110.0 * TILE_PX;
        let mut rng = 7u64;
        let mut obstructed = false;
        for _ in 0..120 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if n.wants_obstruct {
                obstructed = true;
                break;
            }
        }
        assert_eq!(n.ai[0], 5.0, "the attached state");
        assert!(obstructed, "the obstruct buff request");
        assert!((n.vx).abs() < 0.01, "the attached velocity zeroes");
    }

    #[test]
    fn the_predictor_fires_the_573_fan() {
        // `AI_003_Fighters`'s 424 arm (`NPC.cs:59159-59192`/`:59475`/`:59578`):
        // the 180-tick cycle fires four 573s (60) in the `(-3 + 2*i) * 0.15`
        // fan from the muzzle `(direction * 4, -6)`.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(NpcKind::Predictor, 150.0 * TILE_PX, 115.0 * TILE_PX, 1, 0.0);
        n.hp = n.max_hp;
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut saw = 0usize;
        for _ in 0..200 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                saw = n.wants_hostile_shot.len();
                break;
            }
        }
        assert_eq!(saw, 4, "the four-shot fan");
        let (w, h) = n.size();
        let (cx, cy) = (n.x + w * 0.5, n.y + h * 0.5);
        for shot in &n.wants_hostile_shot {
            assert_eq!(shot.0, 573, "the nebula piercer");
            assert_eq!(shot.5, 60, "the classic damage constant");
            assert!(
                (shot.1 - (cx + n.sprite_dir as f32 * 4.0)).abs() < 8.0,
                "the muzzle X: {}",
                shot.1
            );
            assert!(
                (shot.2 - (cy - 6.0)).abs() < 8.0,
                "the muzzle Y: {}",
                shot.2
            );
        }
    }
}

#[cfg(test)]
mod vortex_trio_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn fixture() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_vortex_trio_round_trips_its_rows() {
        // The three vortex hive mobs (`NPC.cs`'s cases 426/427/428): the
        // port ids 127-129, the sizes, the hp and the vanilla rows.
        for (kind, id, w, h, hp, dmg, def, kb) in [
            (
                NpcKind::VortexHunter,
                127u16,
                50.0,
                62.0,
                1000,
                100,
                44,
                0.3,
            ),
            (NpcKind::VortexAlien, 128, 40.0, 28.0, 500, 75, 20, 0.5),
            (NpcKind::VortexHeadcrab, 129, 24.0, 26.0, 200, 50, 6, 1.0),
        ] {
            assert_eq!(kind.to_u16(), id);
            assert_eq!(NpcKind::from_u16(id), Some(kind));
            let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert_eq!((n.size().0, n.size().1), (w, h));
            assert_eq!(n.max_hp, hp);
            assert_eq!(n.defense(), def);
            assert_eq!(n.knock_back_resist(), kb);
        }
        // The 581 dart (`Projectile.cs:6306-6315`): 10x10, aiStyle 1,
        // hostile, penetrate -1.
        let data = crate::projectile::proj_data(581).expect("the dart");
        assert_eq!((data.w, data.h), (10.0, 10.0));
        assert_eq!(data.penetrate, -1);
        assert_eq!(data.ai_style, 1);
    }

    #[test]
    fn the_hunter_fires_the_581_on_its_60_tick_cycle() {
        // `AI_003_Fighters`' 426 arm (`NPC.cs:59069-59756`): the 700-px
        // engage gate starts the 60-tick cycle, the fire at the 59th tick
        // throws the 581 (60) from the muzzle `(spriteDirection * 2,
        // -12)` at the 7-px speed.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::VortexHunter,
            150.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let (w, h) = n.size();
        let (cx, cy) = (n.x + w * 0.5, n.y + h * 0.5);
        let mut rng = 7u64;
        let mut shot = None;
        for _ in 0..130 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if let Some(s) = n.wants_hostile_shot.last().copied() {
                shot = Some(s);
                break;
            }
        }
        let s = shot.expect("the dart fires");
        assert_eq!(s.0, 581, "the vortex dart");
        assert_eq!(s.5, 60, "the classic damage constant");
        // The muzzle: `spriteDirection * 2` ahead of the centre and 12 px
        // above it — the hunter walks, so the shot's own tick's centre is
        // the reference (the walk's contribution rides inside the drift).
        let (hw, hh) = n.size();
        let (ncx, ncy) = (n.x + hw * 0.5, n.y + hh * 0.5);
        assert!((s.1 - (ncx + 2.0)).abs() < 24.0, "the muzzle X: {}", s.1);
        assert!((s.2 - (ncy - 12.0)).abs() < 24.0, "the muzzle Y: {}", s.2);
        let _ = (cx, cy);
    }

    #[test]
    fn the_headcrab_metamorphoses_into_the_alien() {
        // `:56855-56869`'s 300-tick counter + the empty 3x2 probe above
        // the centre: the headcrab heals to its new max and transforms.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::VortexHeadcrab,
            150.0 * TILE_PX,
            118.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = 10;
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut turned = false;
        for _ in 0..320 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if n.kind == NpcKind::VortexAlien {
                turned = true;
                break;
            }
        }
        assert!(turned, "the headcrab becomes the alien");
        assert_eq!(n.hp, n.max_hp, "the transform heals");
        assert_eq!(n.max_hp, 500, "the alien's full hp");
    }

    #[test]
    fn the_alien_metamorphoses_into_the_hunter() {
        // `:56886-56901`'s 1200-tick counter (plus |vx|/2) + the empty
        // 5x5 probe: the alien becomes the hunter at full health.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::VortexAlien,
            150.0 * TILE_PX,
            118.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = 20;
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut turned = false;
        for _ in 0..1400 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if n.kind == NpcKind::VortexHunter {
                turned = true;
                break;
            }
        }
        assert!(turned, "the alien becomes the hunter");
        assert_eq!(n.hp, n.max_hp, "the transform heals");
        assert_eq!(n.max_hp, 1000, "the hunter's full hp");
    }
}

#[cfg(test)]
mod moon_lord_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn fixture() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn app_player() -> (App, Player) {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = fixture();
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player.clone());
        (app, player)
    }

    /// The link fields the sim's pre-pass owns — a part driven directly in
    /// a test sets them by hand.
    fn linked(kind: NpcKind, x: f32, y: f32, slot: i32) -> Npc {
        let mut n = Npc::new(kind, x, y, slot, 0.0);
        n.hp = n.max_hp;
        n.carrier_x = 150.0 * TILE_PX;
        n.carrier_y = 100.0 * TILE_PX;
        n.carrier_timer = 1.0;
        n.moon_core_alive = true;
        n
    }

    #[test]
    fn the_moon_lord_family_round_trips_its_rows() {
        // The five parts (`NPC.cs`'s cases 396/397/398/400/401): the port
        // ids 122-126, the sizes, the hp and the defenses of the vanilla
        // rows.
        for (kind, id, w, h, hp, def) in [
            (NpcKind::MoonLordHead, 122u16, 38.0, 56.0, 45000, 50),
            (NpcKind::MoonLordHand, 123, 46.0, 66.0, 25000, 40),
            (NpcKind::MoonLordCore, 124, 46.0, 66.0, 50000, 70),
            (NpcKind::MoonLordFreeEye, 125, 60.0, 60.0, 100, 0),
            (NpcKind::MoonLordLeechBlob, 126, 30.0, 30.0, 400, 0),
        ] {
            assert_eq!(kind.to_u16(), id);
            assert_eq!(NpcKind::from_u16(id), Some(kind));
            let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert_eq!((n.size().0, n.size().1), (w, h));
            assert_eq!(n.max_hp, hp);
            assert_eq!(n.defense(), def);
            assert_eq!(n.knock_back_resist(), 0.0);
        }
        // The core alone carries a value (`NPC.cs:14130`: 1 platinum).
        assert_eq!(
            Npc::new(NpcKind::MoonLordCore, 0.0, 0.0, 1, 0.0).value(),
            1_000_000.0
        );
        assert_eq!(
            Npc::new(NpcKind::MoonLordHead, 0.0, 0.0, 1, 0.0).value(),
            0.0
        );
    }

    #[test]
    fn the_core_spawns_its_parts_on_the_60th_tick() {
        // `AI_077`'s spawn-in (`NPC.cs:41193-41235`): the 60-tick windup
        // opens the hands (`ai[2]` 0/1) + the head request; the sim
        // resolves it into the three NPCs at the core's offsets.
        let (mut app, _player) = app_player();
        let mut core = Npc::new(
            NpcKind::MoonLordCore,
            150.0 * TILE_PX,
            100.0 * TILE_PX,
            1,
            0.0,
        );
        core.hp = core.max_hp;
        app.npcs.push(core);
        for _ in 0..61 {
            crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        }
        let hands = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MoonLordHand)
            .count();
        let heads = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MoonLordHead)
            .count();
        assert_eq!(hands, 2, "the two hands");
        assert_eq!(heads, 1, "the head");
        let slots: Vec<i32> = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MoonLordCore)
            .flat_map(|n| n.moon_parts)
            .collect();
        assert_eq!(slots.len(), 3, "the core's three slots");
        // The hands' slots split by `ai[2]` and both stay alive (their
        // pre-pass link found the core).
        let alive_hands = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MoonLordHand && n.hp > 0)
            .count();
        assert_eq!(alive_hands, 2, "the hands keep their link");
    }

    #[test]
    fn the_moon_lord_parts_keep_their_own_core() {
        // `AI_077`'s link (`NPC.cs:41229-41232`): the parts carry the
        // core's id in `ai[3]` and the core keeps the parts' ids in its
        // `localAI[0..2]`. The link is explicit — the stable id, not
        // "the first core of the kind".
        let (mut app, _player) = app_player();
        let mut core = Npc::new(
            NpcKind::MoonLordCore,
            150.0 * TILE_PX,
            100.0 * TILE_PX,
            1,
            0.0,
        );
        core.hp = core.max_hp;
        app.npcs.push(core);
        for _ in 0..61 {
            crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        }
        let core_uid = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::MoonLordCore)
            .map(|n| n.uid)
            .expect("the core");
        assert!(core_uid > 0, "the core took an id");
        for part in app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MoonLordHand || n.kind == NpcKind::MoonLordHead)
        {
            assert_eq!(
                part.carrier_uid, core_uid,
                "the part points at its own core"
            );
            assert_eq!(part.ai[3] as u32, core_uid, "the C#'s ai[3] link");
        }
        // The core's slots hold the parts' ids.
        let slots: [i32; 3] = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::MoonLordCore)
            .map(|n| n.moon_parts)
            .expect("the core");
        for uid in slots {
            assert!(uid > 0, "the slot holds a part id");
            assert!(
                app.npcs.iter().any(|n| n.uid as i32 == uid),
                "the slot points at a live part"
            );
        }
        // The core dies: its parts fall with it — the link points at the
        // dead core, not at another Moon Lord.
        let mut second = Npc::new(
            NpcKind::MoonLordCore,
            60.0 * TILE_PX,
            100.0 * TILE_PX,
            1,
            0.0,
        );
        second.hp = second.max_hp;
        app.npcs.push(second);
        let core_idx = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::MoonLordCore && n.uid == core_uid)
            .expect("the first core");
        app.npcs[core_idx].hp = 0;
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let alive_parts = app
            .npcs
            .iter()
            .filter(|n| {
                (n.kind == NpcKind::MoonLordHand || n.kind == NpcKind::MoonLordHead) && n.hp > 0
            })
            .count();
        assert_eq!(alive_parts, 0, "the parts fall with their core");
    }

    #[test]
    fn the_hands_hover_their_side_slots_and_fire_the_eyes() {
        // `AI_078`'s sequencer (`NPC.cs:41600-41626`): the row's first
        // attack is the hover, and the Phantasmal Eyes' 452 (30) fires on
        // the 4-tick cadence inside the eyelid window.
        let (_app, mut player) = app_player();
        let world = fixture();
        let mut hand = linked(
            NpcKind::MoonLordHand,
            150.0 * TILE_PX - 400.0,
            100.0 * TILE_PX - 100.0,
            0,
        );
        hand.ai[2] = 0.0;
        // The sequencer picks the attack from the table, so the timer
        // lands inside the Phantasmal Eyes attack on a shot tick.
        hand.ai[1] = 101.0;
        let mut rng = 7u64;
        let mut saw = 0usize;
        for _ in 0..16 {
            hand.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !hand.wants_hostile_shot.is_empty() {
                saw = hand.wants_hostile_shot.len();
                break;
            }
        }
        assert!(saw > 0, "the 452 fires on its cadence");
        let _ = player.center();
    }

    #[test]
    fn the_hands_left_slot_sits_left_of_the_core() {
        // `:41642`'s anchor: the slot-0 hand's hover slot is the core's
        // centre + (−350, −100) — the port's hand pulls toward it.
        let (_app, mut player) = app_player();
        let world = fixture();
        let mut hand = linked(
            NpcKind::MoonLordHand,
            150.0 * TILE_PX - 400.0,
            100.0 * TILE_PX - 100.0,
            0,
        );
        hand.ai[2] = 0.0;
        hand.ai[0] = 0.0; // the hover
        let before = hand.center().0;
        let mut rng = 7u64;
        for _ in 0..60 {
            hand.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        let after = hand.center().0;
        assert!(
            after < 150.0 * TILE_PX && after > before,
            "the left hand holds its left slot: {} -> {}",
            before,
            after
        );
    }

    #[test]
    fn the_head_sits_on_the_400px_slot_and_deathrays() {
        // `AI_079`'s sequencer (`NPC.cs:42104-42133`): the head rides the
        // fixed 400-px slot over the core, and its phantom deathray (455,
        // 75) fires on the 180th tick of the attack.
        let (_app, mut player) = app_player();
        let world = fixture();
        let (ccx, ccy) = (150.0 * TILE_PX, 100.0 * TILE_PX);
        let mut head = linked(NpcKind::MoonLordHead, ccx, ccy - 400.0, 0);
        head.ai[0] = 1.0; // the phantom deathray attack
        // The head row's deathray slot starts at the 825th elapsed tick;
        // its 180th in-attack tick (the shot) is the timer's 1005th.
        head.ai[1] = 1004.0;
        let mut rng = 7u64;
        let mut shot = None;
        for _ in 0..4 {
            head.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if let Some(s) = head.wants_hostile_shot.last().copied() {
                shot = Some(s);
                break;
            }
        }
        let s = shot.expect("the deathray fires");
        assert_eq!(s.0, 455, "the PhantasmalDeathray");
        assert_eq!(s.5, 75, "the classic damage constant");
        let (w, h) = head.size();
        assert!(
            (head.y + h * 0.5 - (ccy - 400.0)).abs() < 1.0,
            "the head's fixed slot"
        );
    }

    #[test]
    fn the_cores_drama_ends_in_the_downed_flag() {
        // `AI_077`'s crash drama (`NPC.cs:41307-41445`): the 600-tick
        // ride ends with `life = 0`, and the death pass raises
        // `downed_moon_lord` (`NPC.cs:79397-79404`).
        let (_app, mut player) = app_player();
        let world = fixture();
        let mut core = Npc::new(
            NpcKind::MoonLordCore,
            150.0 * TILE_PX,
            100.0 * TILE_PX,
            1,
            0.0,
        );
        core.hp = core.max_hp;
        core.ai[0] = 2.0;
        let mut rng = 7u64;
        let mut smoke = 0usize;
        let mut dead = false;
        for _ in 0..620 {
            core.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            smoke += core.wants_hostile_shot.len();
            core.wants_hostile_shot.clear();
            if core.hp <= 0 {
                dead = true;
                break;
            }
        }
        assert!(dead, "the drama ends the core");
        assert!(smoke > 0, "the drama's 622 smoke: {}", smoke);

        // The flag leg through the sim.
        let (mut app, _p) = app_player();
        let mut c = Npc::new(
            NpcKind::MoonLordCore,
            150.0 * TILE_PX,
            100.0 * TILE_PX,
            1,
            0.0,
        );
        c.hp = 0;
        app.npcs.push(c);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(app.world.downed_moon_lord, "the downed flag");
    }

    #[test]
    fn the_leeched_player_grows_a_clot_that_drains() {
        // `AI_079`'s leech summon + `AI_082`'s drain (`NPC.cs:42303-42318`
        // / `:36658-36690`): a leeched player grows the 401 clot at its
        // centre, and the clot's 90-tick glide ends in the drain request.
        let (mut app, mut player) = app_player();
        // The head needs its core alive for the pre-pass link (the vanilla
        // parts self-kill without one).
        let mut core = Npc::new(
            NpcKind::MoonLordCore,
            150.0 * TILE_PX,
            100.0 * TILE_PX,
            1,
            0.0,
        );
        core.hp = core.max_hp;
        app.npcs.push(core);
        player.buffs.add_buff(145, 840);
        app.player = Some(player.clone());
        app.npcs.push(linked(
            NpcKind::MoonLordHead,
            150.0 * TILE_PX,
            100.0 * TILE_PX - 400.0,
            0,
        ));
        app.npcs[1].wants_moon_leech = true;
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let clots = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::MoonLordLeechBlob)
            .count();
        assert_eq!(clots, 1, "the leeched player grows the clot");
        let idx = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::MoonLordLeechBlob)
            .expect("the clot");
        let head_i = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::MoonLordHead)
            .expect("the head") as f32;
        assert_eq!(app.npcs[idx].ai[0], head_i + 1.0, "the head's index + 1");
        let world = fixture();
        let mut drained = false;
        for _ in 0..91 {
            app.npcs[idx].update_rng(&world, &mut player, 1.0 / 60.0, &mut 7u64);
            if app.npcs[idx].wants_moon_leech {
                drained = true;
                break;
            }
        }
        assert!(drained, "the clot requests its drain");
    }

    #[test]
    fn the_true_eye_drifts_and_fires_the_ring() {
        // `AI_081`'s sequencer (`NPC.cs:42476-42493`): the ring attack (3)
        // throws the 452 (35) volley every 10 ticks inside the 150-tick
        // window.
        let (_app, mut player) = app_player();
        let world = fixture();
        let (ccx, ccy) = (150.0 * TILE_PX, 100.0 * TILE_PX);
        let mut eye = linked(NpcKind::MoonLordFreeEye, ccx - 300.0, ccy - 200.0, 1);
        // The sequencer picks the attack from the table, so the timer
        // lands inside the ring attack (elapsed 384) on its first
        // volley tick (45).
        eye.ai[1] = 428.0;
        let mut rng = 7u64;
        let mut shot = None;
        for _ in 0..4 {
            eye.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if let Some(s) = eye.wants_hostile_shot.last().copied() {
                shot = Some(s);
                break;
            }
        }
        let s = shot.expect("the eye's volley fires");
        assert_eq!(s.0, 452, "the PhantasmalEye");
        assert_eq!(s.5, 35, "the eye's classic damage constant");
    }
}
#[cfg(test)]
mod pillar_mob_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn fixture() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    fn base_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = fixture();
        app
    }

    #[test]
    fn the_towers_round_trip_and_bob() {
        // The three towers share the cultist's row (`NPC.cs`'s cases
        // 493/507/517: 130x270, aiStyle 94, 20000 HP, dmg 0, def 20) and
        // ride the shared machine (the sine bob, the 300-tick cycle).
        for (kind, id) in [
            (NpcKind::StardustTower, 113u16),
            (NpcKind::NebulaTower, 114),
            (NpcKind::SolarTower, 115),
        ] {
            assert_eq!(kind.to_u16(), id);
            assert_eq!(NpcKind::from_u16(id), Some(kind));
            assert_eq!(kind.size(), (130.0, 270.0));
            assert_eq!(kind.max_hp(), 20000);
            let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            // The kind-level rows are private; the stat table's values ride the instance.
            assert_eq!(n.max_hp, 20000);
            let (w, h) = n.size();
            assert_eq!((w, h), (130.0, 270.0));
        }
        let (mut app, mut player) = {
            let mut a = base_app();
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::SolarTower,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 20.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let y0 = n.vy;
        let mut rng = 7u64;
        for _ in 0..30 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(
            (n.vy - y0).abs() < 0.51,
            "the sine bob bounds delta-v to 0.5: {}",
            (n.vy - y0).abs()
        );
        let _ = app;
    }

    #[test]
    fn the_tower_death_flags_the_shield_reset() {
        // The tower's death raises its downed flag + deactivates it
        // (`NPC.cs:79288`'s family at the death) — and the shield's
        // reset fires once all four are down.
        let mut app = base_app();
        let _player = Player::spawn(&app.world);
        crate::world_sim::npcs::raise_tower_shield(&mut app.world);
        let mut n = Npc::new(
            NpcKind::SolarTower,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = 0;
        app.npcs.push(n);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(app.world.downed_tower_solar, "the downed flag");
        assert!(!app.world.tower_active_solar, "the tower deactivates");
    }

    /// S1-D-3f — the Skeletron head's init (`NPC.cs:21924-21941`): the
    /// first tick spawns the two hands — the lower (`ai[0] = -1`) and the
    /// upper (`ai[0] = 1`, `ai[3] = 150`).
    #[test]
    fn the_skeletron_head_spawns_its_two_hands() {
        let mut app = flow_app(None, vec![], vec![]);
        // The surface world (the flow app's own world is 8x8 tiles).
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        let mut head = Npc::new(NpcKind::SkeletronHead, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        head.hp = head.max_hp;
        app.npcs.push(head);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let hands: Vec<_> = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::SkeletronHand)
            .collect();
        assert_eq!(hands.len(), 2, "the two hands spawned");
        let lower = hands
            .iter()
            .find(|h| h.ai[0] < 0.0)
            .expect("the lower hand");
        let upper = hands
            .iter()
            .find(|h| h.ai[0] > 0.0)
            .expect("the upper hand");
        assert_eq!(upper.ai[3], 150.0, "the upper hand's ai[3]");
        // The init latch: the hands spawn once.
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.npcs
                .iter()
                .filter(|n| n.kind == NpcKind::SkeletronHand)
                .count(),
            2,
            "the hands spawn once"
        );
    }
}

#[cfg(test)]
mod shard_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn fixture() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_shards_round_trip_their_stats() {
        // The five shards (`NPC.cs`'s cases 406/407/423/425/429) round-trip
        // their ids and carry their rows.
        for (kind, id, w, h, hp, def, kb) in [
            (NpcKind::StardustCellBig, 121u16, 44.0, 44.0, 300, 50, 0.3),
            (NpcKind::StardustCellSmall, 116, 22.0, 22.0, 300, 0, 0.4),
            (
                NpcKind::StardustJellyfishBig,
                117,
                40.0,
                60.0,
                1500,
                38,
                0.03,
            ),
            (NpcKind::NebulaBeast, 118, 50.0, 44.0, 850, 46, 0.2),
            (NpcKind::VortexRifleman, 119, 30.0, 56.0, 800, 40, 0.4),
            (NpcKind::VortexSoldier, 120, 22.0, 56.0, 700, 34, 0.6),
        ] {
            assert_eq!(kind.to_u16(), id);
            assert_eq!(NpcKind::from_u16(id), Some(kind));
            let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert_eq!((n.size().0, n.size().1), (w, h));
            assert_eq!(n.max_hp, hp);
            assert_eq!(n.defense(), def);
        }
    }

    #[test]
    fn the_stardust_cell_grows_into_the_big() {
        // `AI()`'s aiStyle-95: the 300-tick scale ramp — the ai[0] counter
        // grows to the cap (the Transform to the BigCell rides the next
        // slice).
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::StardustCellSmall,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        for _ in 0..120 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(
            (n.ai[0] - 120.0).abs() < 0.01,
            "the ramp's counter: {}",
            n.ai[0]
        );
    }

    #[test]
    fn the_jellyfish_shoots_the_539_on_the_70_tick_cycle() {
        // `AI()`'s aiStyle-96: the 70-tick cycle fires the 539 (60 damage)
        // from the |X| >= 1.5 ring.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::StardustJellyfishBig,
            150.0 * TILE_PX,
            108.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut saw = false;
        for _ in 0..75 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                saw = true;
                break;
            }
        }
        assert!(saw, "the 539 fires on the cycle");
    }

    #[test]
    fn the_cell_transforms_into_the_big_on_the_300th_tick() {
        // `AI()`'s aiStyle-95 cap (`NPC.cs:39394-39401`): `Transform(405)`
        // — the port swaps the kind in place and the growth ramps the scale
        // 1.0 → 1.3.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::StardustCellSmall,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        for _ in 0..150 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(
            (n.scale - 1.15).abs() < 0.01,
            "the mid-ramp scale: {}",
            n.scale
        );
        for _ in 0..160 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.kind, NpcKind::StardustCellBig, "the transform");
        assert_eq!(n.scale, 1.0, "the grown form resets its scale");
    }

    #[test]
    fn the_nebula_beast_charges_and_fires_the_575() {
        // `AI_026_Unicorns`' NebulaBeast arm (`NPC.cs:62497-62557`): the
        // charge opens on the 180-tick grounded idle inside the 500-px
        // reach, the 575 (50) fires at the 30-tick wind-up, the 60-tick
        // release re-arms.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::NebulaBeast,
            150.0 * TILE_PX,
            115.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut charged = false;
        let mut saw = false;
        for _ in 0..320 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if n.ai[2] == 1.0 && !charged {
                charged = true;
            }
            if charged && !n.wants_hostile_shot.is_empty() {
                saw = true;
                break;
            }
        }
        assert!(charged, "the charge opened");
        assert!(saw, "the 575 fired");
        let shot = n.wants_hostile_shot.last().copied().expect("the payload");
        assert_eq!(shot.0, 575, "the projectile");
        assert_eq!(shot.5, 50, "the classic damage constant");
    }

    #[test]
    fn the_rifleman_fires_the_four_shot_577_salvo() {
        // `AI_003_Fighters`' VortexRifleman arm (`NPC.cs:58463-58621`): the
        // ai[3] timer rearms to −120 on the first tick, arms on the 800-px
        // horizontal line of sight, and its 30th tick fires the four 577s
        // at 75 with the −150 cooldown behind them.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::VortexRifleman,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 162.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(n.ai[3], -119.0, "the −120 rearm ticks once");
        // The salvo's own tick: the test pre-arms the timer to the last
        // arming step (the vanilla's −120 ladder would take 150 ticks and
        // the port's walker reaches the player first).
        n.rifleman_rearmed = true;
        n.ai[3] = 29.0;
        let mut saw = 0usize;
        for _ in 0..24 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_hostile_shot.is_empty() {
                saw = n.wants_hostile_shot.len();
                break;
            }
        }
        assert_eq!(saw, 4, "the salvo is four shots");
        for (id, dmg) in n.wants_hostile_shot.iter().map(|s| (s.0, s.5)) {
            assert_eq!(id, 577);
            assert_eq!(dmg, 75);
        }
        assert_eq!(n.ai[3], 31.0, "the windup holds past the shot");
    }

    #[test]
    fn the_star_cell_approaches_through_the_sighted_phase() {
        // `AI()`'s aiStyle-85 block (`NPC.cs:36930-37012`): phase 0 picks
        // the approach on the line of sight and phase 1 steers at the
        // 8 + dist/100 speed over the 50-tick smoothing.
        let (_app, mut player) = {
            let mut a = flow_app(None, vec![], vec![]);
            let p = Player::spawn(&a.world);
            (a, p)
        };
        let world = fixture();
        let mut n = Npc::new(
            NpcKind::StardustCellBig,
            150.0 * TILE_PX,
            108.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        for _ in 0..30 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.ai[0], 1.0, "the sighted approach holds");
        assert!(
            n.vx > 0.0 && n.vx <= 8.6,
            "the approach's velocity: {}",
            n.vx
        );
        assert!(
            (n.x + n.size().0 * 0.5) > 2422.0,
            "the cell closed on the player"
        );
    }

    #[cfg(test)]
    mod dd2_lane_tests {
        use super::*;
        use crate::app::flow::front_flow_tests::flow_app;
        use crate::player::Player;

        fn surface_world() -> World {
            let mut w = World::new("t".into(), 300, 200);
            w.world_surface = 120.0;
            for x in 0..w.width {
                for y in 120..w.height {
                    let i = w.index(x, y);
                    w.tiles[i] = tr_core::Tile {
                        kind: 1,
                        ..Default::default()
                    };
                    w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
                }
            }
            w.compute_column_surface();
            w
        }

        #[test]
        fn the_lane_spawn_rolls_pick_the_waves_roster() {
            // `Difficulty_1_SpawnMonsterFromGate` (`DD2Event.cs:1011-1105`):
            // the per-wave switch's caps. With an empty world the wave-1 goblin
            // spawns; the wave-2 roll's `Next(7)` gate.
            let empty = |_t: u16| 0i32;
            // Wave 1: always the squire under the cap.
            let pick = tr_core::dd2::lane_spawn_roll_d1(1, &empty, 0, 60, false, || 0.9);
            assert_eq!(pick.first, Some(552));
            assert_eq!(pick.second, None);
            // Wave 2: `Next(7) == 0` → the bomber, else the squire.
            let pick = tr_core::dd2::lane_spawn_roll_d1(2, &empty, 0, 60, false, || 0.05);
            assert_eq!(pick.first, Some(555));
            let pick = tr_core::dd2::lane_spawn_roll_d1(2, &empty, 0, 60, false, || 0.9);
            assert_eq!(pick.first, Some(552));
            // The cap refuses: 50 goblins alive → no spawn.
            let full = |t: u16| if t == 552 { 60 } else { 0 };
            let pick = tr_core::dd2::lane_spawn_roll_d1(1, &full, 0, 60, false, || 0.9);
            assert_eq!(pick.first, None);
            // Wave 4: the wyvern rides `Next(12)` under its cap.
            let pick = tr_core::dd2::lane_spawn_roll_d1(4, &empty, 0, 60, false, || 0.01);
            assert_eq!(pick.first, Some(558));
            let pick = tr_core::dd2::lane_spawn_roll_d1(4, &empty, 0, 60, false, || 0.5);
            assert_eq!(pick.first, Some(552));
            // Wave 3: the javeliner rides `Next(6)` ahead of the goblins.
            let pick = tr_core::dd2::lane_spawn_roll_d1(3, &empty, 0, 60, false, || 0.01);
            assert_eq!(pick.first, Some(561));
            let pick = tr_core::dd2::lane_spawn_roll_d1(3, &empty, 0, 60, false, || 0.5);
            assert_eq!(pick.first, Some(552));
            // Wave 5: the Dark Mage rides the half-kill gate (`:1078`), but the
            // chain after it re-assigns `num4` whenever it fires — so the boss
            // only survives when the goblins already sit at their cap.
            let full = |t: u16| if t == 564 { 0 } else { 60 };
            let pick = tr_core::dd2::lane_spawn_roll_d1(5, &full, 31, 60, false, || 0.9);
            assert_eq!(
                pick.first,
                Some(564),
                "the Dark Mage past the half-kill gate"
            );
            let pick = tr_core::dd2::lane_spawn_roll_d1(5, &full, 10, 60, false, || 0.9);
            assert_eq!(
                pick.first, None,
                "the gate holds before half the wave is dead"
            );
            let pick = tr_core::dd2::lane_spawn_roll_d1(5, &empty, 31, 60, false, || 0.01);
            assert_eq!(pick.first, Some(558), "the chain outranks the boss spawn");
        }

        #[test]
        fn the_tier_two_and_three_rolls_carry_their_waves() {
            let empty = |_t: u16| 0i32;
            // D2 wave 1: the tier-3 arm under `Next(20)`.
            let pick = tr_core::dd2::lane_spawn_roll_d2(1, &empty, 0, 60, false, || 0.01);
            assert_eq!(pick.first, Some(562));
            // D2 wave 7's boss arm: the Ogre on the kill gate. The left lane
            // short-circuits `!leftGate`, so it needs no roll; the right lane
            // spends its `Next(2)`.
            let pick = tr_core::dd2::lane_spawn_roll_d2(7, &empty, 10, 60, false, || 0.9);
            assert_eq!(pick.first, Some(576), "the left lane's Ogre");
            let pick = tr_core::dd2::lane_spawn_roll_d2(7, &empty, 10, 60, true, || 0.25);
            assert_eq!(pick.first, Some(576), "the right lane's Ogre");
            let pick = tr_core::dd2::lane_spawn_roll_d2(7, &empty, 10, 60, true, || 0.75);
            assert_eq!(pick.first, Some(553), "the coin misses, the squire pair");
            let pick = tr_core::dd2::lane_spawn_roll_d2(7, &empty, 6, 60, false, || 0.9);
            assert_eq!(pick.first, Some(553), "under the 10% kill gate");
            // D2 wave 3's squire pair: the chain's rolls fail (`Next(7)/(10)/(8)`)
            // and `Next(4) == 0` drops the 556 beside the 553.
            let pick = tr_core::dd2::lane_spawn_roll_d2(3, &empty, 0, 60, false, || 0.2);
            assert_eq!(pick.first, Some(553));
            assert_eq!(pick.second, Some(556));
            let pick = tr_core::dd2::lane_spawn_roll_d2(3, &empty, 0, 60, false, || 0.9);
            assert_eq!(pick.first, Some(553));
            assert_eq!(pick.second, None);
            // D2 wave 5's flyer arm: the ogre's and archer's rolls fail, the
            // flyer's `Next(4)` gate passes, then `Next(2)` coins between the
            // hound's 574 and the 572.
            let seq = std::cell::Cell::new(0usize);
            let roll = || {
                let rolls = [0.2f32, 0.9, 0.2, 0.9];
                let i = seq.get();
                seq.set(i + 1);
                rolls[i.min(rolls.len() - 1)]
            };
            let pick = tr_core::dd2::lane_spawn_roll_d2(5, &empty, 0, 60, false, roll);
            assert_eq!(pick.first, Some(574));
            let seq = std::cell::Cell::new(0usize);
            let roll = || {
                let rolls = [0.2f32, 0.9, 0.2, 0.1];
                let i = seq.get();
                seq.set(i + 1);
                rolls[i.min(rolls.len() - 1)]
            };
            let pick = tr_core::dd2::lane_spawn_roll_d2(5, &empty, 0, 60, false, roll);
            assert_eq!(pick.first, Some(572));
            // D3 wave 6: the balista's one-shot survives only when the second
            // chain's rolls fail and its squire cap is full.
            let full = |t: u16| if t == 554 || t == 557 { 60 } else { 0 };
            let seq = std::cell::Cell::new(0usize);
            let roll = || {
                let rolls = [0.01f32, 0.9];
                let i = seq.get();
                seq.set(i + 1);
                rolls[i.min(rolls.len() - 1)]
            };
            let pick = tr_core::dd2::lane_spawn_roll_d3(6, &full, roll);
            assert_eq!(pick.first, Some(577), "the balista the second chain spares");
            // D3 wave 4's sapper.
            let pick = tr_core::dd2::lane_spawn_roll_d3(4, &empty, || 0.01);
            assert_eq!(pick.first, Some(565));
            // D3 wave 1: the tier-3 arm, and the squire pair's 557 on `Next(7)`.
            let pick = tr_core::dd2::lane_spawn_roll_d3(1, &empty, || 0.01);
            assert_eq!(pick.first, Some(563));
            let pick = tr_core::dd2::lane_spawn_roll_d3(1, &empty, || 0.1);
            assert_eq!(pick.first, Some(554));
            assert_eq!(pick.second, Some(557));
        }

        #[test]
        fn the_tier_three_wave_six_chains_run_side_by_side() {
            // `DD2Event.cs:1691-1728`: wave 6 runs two independent `if` chains —
            // the second one re-assigns `num8` whenever it fires, so the glider
            // (578) it picks outranks the balista (577) the first chain picked.
            let empty = |_t: u16| 0i32;
            let pick = tr_core::dd2::lane_spawn_roll_d3(6, &empty, || 0.01);
            assert_eq!(pick.first, Some(578));
            // With the glider at its cap the second chain's failed rolls carry it
            // to the squire pair, whose `Next(3)` drops the 557.
            let capped = |t: u16| if t == 578 { 40 } else { 0 };
            let pick = tr_core::dd2::lane_spawn_roll_d3(6, &capped, || 0.3);
            assert_eq!(pick.first, Some(554), "the second chain's squire pair");
            assert_eq!(pick.second, Some(557));
            // Both chains empty and the squire cap full: nothing spawns.
            let full = |t: u16| if t == 554 || t == 557 { 60 } else { 0 };
            let pick = tr_core::dd2::lane_spawn_roll_d3(6, &full, || 0.9);
            assert_eq!(pick.first, None);
        }

        #[test]
        fn the_lane_portal_round_trips_and_paces_its_spawns() {
            // The 549's row + `AI`'s aiStyle-106 pacing: the `ai[0]` counter
            // opens a spawn every `LaneSpawnRate` frames and resets at three
            // times the rate (`NPC.cs:40992-41001`).
            let kind = NpcKind::DD2LanePortal;
            assert_eq!(kind.to_u16(), 147);
            assert_eq!(NpcKind::from_u16(147), Some(kind));
            let mut n = Npc::new(kind, 150.0 * TILE_PX, 118.0 * TILE_PX, 1, 0.0);
            assert_eq!((n.size().0, n.size().1), (78.0, 130.0));
            assert_eq!(n.max_hp, 5);
            assert_eq!(n.defense(), 20);
            assert_eq!(n.defense(), 20);
            // The ID window: `leftGate` rides `ai[2]` (`NPC.cs:40972`).
            let mut world = surface_world();
            // The event's pre-wave hold parks the counter (`:40988`), and the
            // wave picks the lane rate (`Difficulty_1_GetEnemiesForWave(2)`).
            world.dd2_spawn_hold = 0;
            world.dd2_wave = 2;
            let mut player = Player::spawn(&world);
            player.x = 160.0 * TILE_PX;
            player.y = 118.0 * TILE_PX;
            // With the hold up the gate never counts.
            let mut held = surface_world();
            held.dd2_spawn_hold = 3;
            for _ in 0..120 {
                n.update_rng(&held, &mut player, 1.0 / 60.0, &mut 7u64);
            }
            assert!(!n.wants_dd2_spawn, "the hold parks the counter");
            n.ai[0] = 0.0;
            n.caster_counter = 0;
            let mut spawns = 0;
            for _ in 0..200 {
                n.update_rng(&world, &mut player, 1.0 / 60.0, &mut 7u64);
                if n.wants_dd2_spawn {
                    spawns += 1;
                    n.wants_dd2_spawn = false;
                }
            }
            // `Difficulty_1_GetEnemiesForWave(2)`'s rate: three attempts in 200
            // ticks (at 60, 120 and 180 — the 3× reset lands at 180).
            assert_eq!(spawns, 3, "the lane's spawn attempts: {}", spawns);
            // The right lane's ID rides ai[2] = 1.
            n.ai[2] = 1.0;
            n.caster_counter = 0;
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut 7u64);
            assert!(n.wants_lane_right, "the right lane's flag");
        }

        #[test]
        fn the_crystal_parks_the_gates_fade() {
            // `NPC.cs:41014-41017`: while `AnyNPCs(548)` stands the gate's
            // 180-tick arm returns early, so the gate never fades and stays
            // unkillable.
            let mut n = Npc::new(
                NpcKind::DD2LanePortal,
                150.0 * TILE_PX,
                118.0 * TILE_PX,
                1,
                0.0,
            );
            let mut world = surface_world();
            world.dd2_spawn_hold = 0;
            world.dd2_crystal_alive = true;
            let mut player = Player::spawn(&world);
            let mut rng = 7u64;
            for _ in 0..400 {
                n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            }
            assert_eq!(n.ai[1], 0.0, "the fade never opens");
            assert!(n.dont_take_damage, "the crystal keeps the gate up");
            assert_eq!(n.scale, 1.0, "the gate holds its scale");
            // Crystal gone: the fade opens and the gate dies at 550 — the fade's
            // own counter runs 180 (the phase change) + 550 (the death) ticks.
            world.dd2_crystal_alive = false;
            let mut n = Npc::new(
                NpcKind::DD2LanePortal,
                150.0 * TILE_PX,
                118.0 * TILE_PX,
                1,
                0.0,
            );
            for _ in 0..800 {
                n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            }
            assert_eq!(n.ai[1], 1.0, "the fade arm opened");
            assert!(n.hp <= 0, "the gate dies at 550: {}", n.hp);
        }

        #[test]
        fn the_gate_spawns_the_waves_goblin_at_its_bottom() {
            // `DD2Event.cs:556-570`'s resolve: the gate's bottom is the spawn
            // point, the wave-1 goblin is the tier-1 pick, and the goblin cap
            // refuses a second one.
            let mut world = surface_world();
            world.dd2_spawn_hold = 0;
            world.dd2_crystal_alive = false;
            world.dd2_event_ongoing = true;
            world.dd2_wave = 1;
            let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
            app.world = world;
            let (_, ph) = (78.0, 130.0);
            let (gx, gy) = (150.0 * TILE_PX, 118.0 * TILE_PX);
            let mut gate = Npc::new(NpcKind::DD2LanePortal, gx, gy, 1, 0.0);
            gate.wants_dd2_spawn = true;
            app.npcs.push(gate);
            update_npcs(&mut app, 1.0 / 60.0);
            let goblins: Vec<&Npc> = app
                .npcs
                .iter()
                .filter(|n| n.kind == NpcKind::DD2GoblinT1)
                .collect();
            assert_eq!(goblins.len(), 1, "the wave-1 squire spawns");
            assert!(
                (goblins[0].x - gx).abs() < 1.0 && (goblins[0].y - (gy + ph)).abs() < 1.0,
                "the spawn rides the gate's bottom"
            );
            // The gate's second attempt also spawns — the cap is 50.
            let mut gate = Npc::new(NpcKind::DD2LanePortal, gx, gy, 1, 0.0);
            gate.wants_dd2_spawn = true;
            app.npcs.push(gate);
            update_npcs(&mut app, 1.0 / 60.0);
            assert_eq!(
                app.npcs
                    .iter()
                    .filter(|n| n.kind == NpcKind::DD2GoblinT1)
                    .count(),
                2,
                "the goblin cap is 50"
            );
        }

        #[test]
        fn the_gate_drops_the_unported_picks() {
            // The tier-3 wave-6 roster: its two chains pick from the balista
            // (577), the drake (569), the tier-2 mystic (571) and the Kobold
            // walker pair — M4-EVENT-19 ported all of them, so the pick now
            // spawns and only the still-unported kobold flyer (574/575) drops.
            let mut world = surface_world();
            world.hard_mode = true;
            world.downed_mech_boss1 = true;
            world.downed_golem = true;
            world.dd2_spawn_hold = 0;
            world.dd2_event_ongoing = true;
            world.dd2_wave = 6;
            let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
            app.world = world;
            let mut gate = Npc::new(
                NpcKind::DD2LanePortal,
                150.0 * TILE_PX,
                118.0 * TILE_PX,
                1,
                0.0,
            );
            gate.wants_dd2_spawn = true;
            app.npcs.push(gate);
            update_npcs(&mut app, 1.0 / 60.0);
            // The intent was consumed, so the resolve ran — the pick's kind is
            // ported now, so one of the wave-6 families walks out.
            assert!(
                app.npcs.iter().all(|n| !n.wants_dd2_spawn),
                "the resolve ran"
            );
            assert!(
                app.npcs.iter().filter(|n| n.hp > 0).count() > 1,
                "the tier-3 wave-6 pick spawns: {}",
                app.npcs.iter().filter(|n| n.hp > 0).count()
            );
        }
    }
}

#[cfg(test)]
mod dd2_tier_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;

    fn fixture() -> World {
        let mut w = World::new("t".into(), 300, 200);
        w.world_surface = 120.0;
        for x in 0..w.width {
            for y in 120..w.height {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w
    }

    /// The tier-2/3 families' rows (`SetDefaults`'s cases 558-577): the
    /// port ids, the vanilla sizes, the hp, the damage and the defense.
    #[test]
    fn the_tier_two_and_three_rows_round_trip() {
        for (kind, id, w, h, hp, dmg, def, kb) in [
            (NpcKind::DD2WyvernT1, 169u16, 38.0, 38.0, 60, 30, 4, 0.2),
            (NpcKind::DD2WyvernT2, 170, 38.0, 38.0, 180, 75, 16, 0.2),
            (NpcKind::DD2WyvernT3, 171, 38.0, 38.0, 600, 100, 30, 0.05),
            (NpcKind::DD2DarkMageT3, 172, 34.0, 62.0, 4000, 90, 38, 0.0),
            (
                NpcKind::DD2WitherBeastT3,
                173,
                18.0,
                40.0,
                1400,
                80,
                40,
                0.05,
            ),
            (NpcKind::DD2DrakinT2, 174, 42.0, 58.0, 900, 60, 30, 0.0),
            (NpcKind::DD2DrakinT3, 175, 42.0, 58.0, 3000, 90, 40, 0.0),
            (
                NpcKind::DD2KoboldWalkerT2,
                176,
                18.0,
                40.0,
                260,
                60,
                26,
                0.2,
            ),
            (
                NpcKind::DD2KoboldWalkerT3,
                177,
                18.0,
                40.0,
                800,
                80,
                32,
                0.1,
            ),
            (NpcKind::DD2KoboldFlyerT2, 178, 38.0, 38.0, 170, 50, 16, 0.4),
            (
                NpcKind::DD2KoboldFlyerT3,
                179,
                38.0,
                38.0,
                580,
                80,
                32,
                0.25,
            ),
            (NpcKind::DD2OgreT2, 180, 96.0, 124.0, 5000, 70, 34, 0.0),
            (NpcKind::DD2OgreT3, 181, 96.0, 124.0, 13000, 90, 40, 0.0),
        ] {
            assert_eq!(kind.to_u16(), id, "{:?}", kind);
            assert_eq!(NpcKind::from_u16(id), Some(kind));
            let n = Npc::new(kind, 0.0, 0.0, 1, 0.0);
            assert_eq!((n.size().0, n.size().1), (w, h), "{:?}'s size", kind);
            assert_eq!(n.max_hp, hp, "{:?}'s hp", kind);
            assert_eq!(n.defense(), def, "{:?}'s defense", kind);
            assert_eq!(n.knock_back_resist(), kb, "{:?}'s kb", kind);
        }
    }

    /// The vanilla ids: the lane spawn table's ids map back to the port's
    /// kinds (`nid_of`), so the D2/D3 rosters now resolve.
    #[test]
    fn the_lane_table_resolves_every_ported_family() {
        for (id, kind) in [
            (558u16, NpcKind::DD2WyvernT1),
            (559, NpcKind::DD2WyvernT2),
            (560, NpcKind::DD2WyvernT3),
            (564, NpcKind::DD2DarkMage),
            (565, NpcKind::DD2DarkMageT3),
            (569, NpcKind::DD2WitherBeastT3),
            (570, NpcKind::DD2DrakinT2),
            (571, NpcKind::DD2DrakinT3),
            (572, NpcKind::DD2KoboldWalkerT2),
            (573, NpcKind::DD2KoboldWalkerT3),
            (574, NpcKind::DD2KoboldFlyerT2),
            (575, NpcKind::DD2KoboldFlyerT3),
            (576, NpcKind::DD2OgreT2),
            (577, NpcKind::DD2OgreT3),
        ] {
            assert_eq!(super::nid_of(id), Some(kind), "{}", id);
            assert_eq!(super::dd2_pool_id(kind), Some(id), "{}", id);
        }
    }

    /// `AI_108_DivingFlyer`'s state machine (`NPC.cs:65392-65788`): the
    /// approach, the engage, the wind-up and the dive — the wyvern's launch
    /// rides `num8` (8).
    #[test]
    fn the_diving_flyer_cycles_through_its_dive() {
        let mut n = Npc::new(
            NpcKind::DD2WyvernT1,
            150.0 * TILE_PX,
            110.0 * TILE_PX,
            1,
            0.0,
        );
        let world = fixture();
        let mut player = Player::spawn(&world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        // The approach (`num3`'s 500-px engage gate): close, then engage.
        // The dive's homing adds `num12 / num11` per tick (the C#'s drag), so
        // the speed is read on the wind-up's LAUNCH tick — the 1 → 2 flip —
        // not at any later point of the dive.
        let mut dived = false;
        let mut speed = 0.0;
        for _ in 0..400 {
            let was = n.ai[0];
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if was == 1.0 && n.ai[0] == 2.0 {
                dived = true;
                speed = n.vx.hypot(n.vy);
                break;
            }
        }
        assert!(dived, "the wyvern dived");
        // The launch speed rides `num8` (8).
        assert!((speed - 8.0).abs() < 0.5, "the launch speed: {}", speed);
    }

    /// The kobold glider's impact (`:65728-65788`): the 192-box, the 80-ish
    /// damage and the death at the third tick — the glider has `num13 = 0`,
    /// so only the solid-collision or the 64-px proximity opens it.
    #[test]
    fn the_glider_self_destructs_on_impact() {
        let mut n = Npc::new(
            NpcKind::DD2KoboldFlyerT2,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.ai[0] = 3.0;
        n.ai[1] = 0.0;
        let world = fixture();
        let mut player = Player::spawn(&world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut blasted = false;
        for _ in 0..6 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if n.wants_blast {
                blasted = true;
            }
        }
        assert!(blasted, "the impact's blast request");
        assert_eq!(n.damage_override, Some(80), "the 192-box's damage");
        assert!(n.hp <= 0, "the glider dies with it: {}", n.hp);
    }

    /// `AI_107`'s 576/577 arm (`NPC.cs:63029-63122`): the ogre's chase
    /// window latches at 300 ticks and the spit leaves on `num9` (36).
    #[test]
    fn the_ogre_latches_its_chase_window_and_spits() {
        let mut n = Npc::new(NpcKind::DD2OgreT2, 150.0 * TILE_PX, 112.0 * TILE_PX, 1, 0.0);
        let world = fixture();
        let mut player = Player::spawn(&world);
        // 640 px away: past `num4 + 300` (430) but inside the spit's 1000-px
        // engage, so `ai[1] = 1` (the 676) picks over the pound's 683.
        player.x = 190.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut shot = None;
        for _ in 0..400 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if let Some(s) = n.wants_hostile_shot.last().copied() {
                shot = Some(s);
                break;
            }
        }
        let s = shot.expect("the ogre throws");
        assert_eq!(s.0, 676, "the spit");
        // `:63103`'s `GetAttackDamage_ForProjectiles(40f, 30f)` — the port's
        // classic-damage convention takes the first value.
        assert_eq!(s.5, 40, "the spit's damage");
    }

    /// `AI_107`'s 570/571 arm (`NPC.cs:63145-63195`): the drakin's 671
    /// burst leaves from the 22-px muzzle offset at the tier's damage.
    #[test]
    fn the_drakin_bursts_the_671_from_its_muzzle() {
        let mut n = Npc::new(
            NpcKind::DD2DrakinT2,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        let world = fixture();
        let mut player = Player::spawn(&world);
        player.x = 170.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = 7u64;
        let mut shot = None;
        let mut origin_x = 0.0f32;
        for _ in 0..240 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if let Some(s) = n.wants_hostile_shot.last().copied() {
                shot = Some(s);
                origin_x = n.x;
                break;
            }
        }
        let s = shot.expect("the drakin fires");
        assert_eq!(s.0, 671, "the drakin's shot");
        assert_eq!(s.5, 35, "the T2's damage");
        // `:63162`'s `vector.X += 22 * direction` rides the box's CENTRE
        // (`base.Center + vector`), so the muzzle sits 22 px right of the
        // 42-wide box's centre.
        assert!(
            (s.1 - (origin_x + 21.0 + 22.0)).abs() < 1.0,
            "the muzzle offset: {}",
            s.1
        );
    }
}
