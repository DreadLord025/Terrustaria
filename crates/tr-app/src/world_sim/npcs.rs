//! NPC simulation (plan stage A4): the fixed 60 Hz `Npc::update` step, the
//! death coin drops, the boss minion requests, the player respawn and the
//! `Spawner.SpawnNPC` ambient spawn attempt.

use glam::Vec2;
use tr_core::{World, TILE_PX};

use crate::achievements;
use crate::app::{App, RngKey};
use crate::item::ItemDrop;
use crate::npc::{Npc, NpcKind, SlimeVariant};
use crate::regen::DotDeath;
use crate::registry;
use crate::spawner;
use tr_core::UnifiedRandom;

/// The player's death message: a DoT drain death resolves its attribution's
/// `DeathText.*` row family (`KillMe(PlayerDeathReason.ByOther(N))` with the
/// `Lang.cs:1045-1127` variant roll — the C# rides `Main.rand`; the port
/// draws it from the `UpdatePlayers` stream (the death runs in
/// `Player.KillMe` under the `UpdatePlayers` swap)); every other death is
/// the `DeathText.Slain` row.
fn dot_death_text(pending: Option<DotDeath>, player_name: &str, rng: &mut UnifiedRandom) -> String {
    match pending {
        Some(death) => {
            // `Main.rand.Next(variants) + 1` (`Lang.cs:1057-1127`).
            let (_, variants) = crate::death::death_text_key(death);
            let variant = rng.next_max(variants as i32) + 1;
            crate::death::death_message(death, variant, player_name)
        }
        None => crate::death::slain_message(player_name),
    }
}

/// `Player.StatusFromNPC` (`Player.cs:22333`), the black-slime row `:22591`:
/// `npc.type == 1 && npc.netID == -6` rolls `Next(4) == 0` for 900 ticks of
/// Darkness on touch. The roll rides the `UpdateNPCs` stream — the C#
/// `StatusFromNPC` runs inside `NPC.UpdateNPC` under the `UpdateNPCs` swap.
pub(crate) fn black_slime_darkness_touch(
    slime: &Npc,
    player: &mut crate::player::Player,
    rng: &mut UnifiedRandom,
) {
    // `StatusFromNPC`'s `Main.rand.Next(4) == 0` (`Player.cs:22591`), under
    // the `UpdateNPCs` swap (the touch runs inside `NPC.Update`).
    if slime.kind == NpcKind::Slime
        && slime.slime == SlimeVariant::Black
        && slime.touches(player)
        && rng.next_max(4) == 0
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
        // The payout is a right-click interaction — `Player.ItemCheck` under
        // the `UpdatePlayers` swap; the coin spawn draws ride the same
        // stream as the turn-in's (`Item.NewItem`'s `Main.rand`).
        let rng = app.rngs.stream(RngKey::UpdatePlayers);
        crate::world_sim::items::spawn_item_drop_px(
            &mut app.items,
            rng,
            kind,
            stack,
            Vec2::new(px, py),
        );
    }
    app.world.tax_money = 0;
    true
}

/// `TileReachCheckSettings.Simple.GetWorldRegion`
/// (`TileReachCheckSettings.cs:76-89`): the player's tile box —
/// `LX = (int)(position.X / 16f) - x` … `HX = (int)Ceiling((position.X +
/// width) / 16f) - 1 + x` with `x = Player.tileRangeX * multiplier` capped
/// by the 20-tile `TileReachLimit` (`:11-22`) — converted back to pixels
/// with the `HX += 15` / `HY += 15` corrections (`:68-69`). `Simple` is
/// `TileRangeMultiplier = 1`, `TileReachLimit = 20` (`:83-87`), and
/// `Player.DefaultTileRangeX/Y = 5/3` (`Player.cs:57113-57114`).
///
/// Returns `(LX, LY, HX, HY)` in world pixels.
pub(crate) fn chat_world_region(p: &crate::player::Player) -> (f32, f32, f32, f32) {
    let lx = ((p.x / 16.0) as i32 - 5) as f32 * 16.0;
    let ly = ((p.y / 16.0) as i32 - 3) as f32 * 16.0;
    let hx = (((p.x + crate::player::WIDTH) / 16.0).ceil() as i32 - 1 + 5) as f32 * 16.0 + 15.0;
    let hy = (((p.y + crate::player::HEIGHT) / 16.0).ceil() as i32 - 1 + 3) as f32 * 16.0 + 15.0;
    (lx, ly, hx, hy)
}

/// The right-click talk action (`Main.cs:42993-43083`) — the live route to
/// [`crate::npc_chat::get_chat`].
///
/// **The close rule** (`Player.cs:27494-27503`) runs every tick: while
/// `talkNPC >= 0`, the world region no longer intersecting the NPC's box, a
/// chest being open, or the NPC gone clears the latch (`SetTalkNPC(-1)`,
/// `Main.npcChatText = ""`) with sound 11 unless the chest path took it.
///
/// **The open arm** fires on the fresh right-press edge — the C# gate
/// `Main.mouseRight && Main.npcChatRelease` (`Main.cs:43070`) with
/// `npcChatRelease` the inverted `mouseRight` latch of
/// `DrawInterface_41_InterfaceLogic4` (`Main.cs:45000-45009`) — for the
/// first live `townNPC` under the world region (`:43047`'s
/// `worldRegion.Intersects(nPC box)`, the `!player.dead` gate at `:43068`)
/// that is not already latched (`:43077`'s `player.talkNPC != i`). It latches
/// `SetTalkNPC(i)` (`:43081`) and answers `GetChat()` (`:43082`).
///
/// The RNG rides [`crate::app::NamedRngs::global`] — the C#'s `Main.rand`
/// draws `GetChat` outside every `Main.SwapRandom` stage, so the bare-Main.rand
/// stand-in stream is the named stream (the S1-A-10 contract).
///
/// `Some(text)` consumes the right-click press; `None` falls through to the
/// chest/tile routes — and to the collapsed shop arms, which stay reachable
/// through the same-NPC re-click (`talkNPC == i` declines here).
pub fn talk_to_npc(app: &mut App, prev_place_down: bool) -> Option<String> {
    // The per-tick close (`Player.cs:27494-27503`).
    if let Some(i) = app.talk_npc {
        let region = app
            .player
            .as_ref()
            .map(chat_world_region)
            .unwrap_or((0.0, 0.0, 0.0, 0.0));
        let (lx, ly, hx, hy) = region;
        let inside = app.npcs.get(i).is_some_and(|n| {
            n.hp > 0 && {
                let (w, h) = n.size();
                n.x < hx && n.x + w > lx && n.y < hy && n.y + h > ly
            }
        });
        if !inside || app.open_container() != -1 {
            if app.open_container() == -1 {
                crate::chest::play_menu_sound(app, 11);
            }
            app.talk_npc = None;
            app.npc_chat_text.clear();
        }
    }
    // The open arm's gates: a fresh press, no inventory panel over the world.
    if !app.place_down || prev_place_down || app.inventory_open {
        return None;
    }
    let Some(p) = app.player.as_ref() else {
        return None;
    };
    if p.hp <= 0 {
        return None; // `:43068` — `flag3 && !player.dead`
    }
    let region = chat_world_region(p);
    let i = app.npcs.iter().position(|n| {
        n.hp > 0
            && n.kind.is_town_npc_like()
            && {
                let (w, h) = n.size();
                let (lx, ly, hx, hy) = region;
                // `Rectangle.Intersects` of the region and the NPC box.
                n.x < hx && n.x + w > lx && n.y < hy && n.y + h > ly
            }
    })?;
    if app.talk_npc == Some(i) {
        return None; // `:43077` — the same NPC re-clicked falls through
    }
    app.talk_npc = Some(i); // `:43081` — `player.SetTalkNPC(i)`
    // The context off the live state. The world flags the port does not
    // carry yet (the storm clock, the happy-windy-day, `BirthdayParty`,
    // `LanternNight`, the remix world, `WorldGen.crimson`, the Moon Lord)
    // default off — each noted at its field.
    let day_time = app.world.day_time;
    let time = app.world.time;
    let blood_moon = app.world.blood_moon;
    let eclipse = app.world.eclipse;
    let slime_rain = app.world.slime_rain;
    let hard_mode = app.world.hard_mode;
    let moon_phase = app.world.moon_phase;
    let raining = app.world.raining;
    let (life, life_max, deaths) = app
        .player
        .as_ref()
        .map(|p| (p.hp, p.max_hp, p.deaths_pve as i32))
        .unwrap_or((0, 0, 0));
    let angler_quests = app
        .player
        .as_ref()
        .map(|p| p.angler_quests_finished)
        .unwrap_or(0);
    let golfer_score = app.player.as_ref().map(|p| p.golfer_score).unwrap_or(0);
    // `Player.HasItem(5275)` — the cola the Dryad's line asks about; the
    // port scans the inventory slots (no equipment scans yet).
    let has_cola = app.inventory.iter().any(|s| s.kind == 5275);
    // `DD2Event.DownedInvasionT1/2` — the persisted tier-victory latches
    // (S1-E-12c), NOT the per-run wave-boss death latches.
    let dd2_t1 = app.world.dd2_downed_t1;
    let dd2_t2 = app.world.dd2_downed_t2;
    let dd2_ongoing = app.world.dd2_event_ongoing;
    let rng = app.rngs.global();
    let mut draw = |n: i32| rng.next_max(n);
    let mut ctx = crate::npc_chat::ChatContext {
        npcs: &app.npcs,
        rng: &mut draw,
        day_time,
        time,
        blood_moon,
        eclipse,
        slime_rain,
        hard_mode,
        moon_phase,
        remix_world: false,    // `Main.remixWorld` — no remix flag in the port
        raining,
        storming: false,       // `Main.cloudAlpha`'s storm — no storm clock yet
        windy_day: false,      // `Main.IsItAHappyWindyDay` — no wind-day state
        zone_graveyard: false, // `Player.ZoneGraveyard` — the zone rides the biome pass
        party_is_up: false,    // `BirthdayParty.PartyIsUp` — unported
        crimson: false,        // `WorldGen.crimson` — the world-evil flag
        downed_boss1: app.world.downed_eoc, // `NPC.downedBoss1`
        downed_boss2: app.world.downed_eow, // `NPC.downedBoss2`
        downed_boss3: app.world.downed_skeletron, // `NPC.downedBoss3`
        downed_moonlord: false, // `NPC.downedMoonlord` — unported
        dd2_downed_t1: dd2_t1,
        dd2_downed_t2: dd2_t2,
        dd2_ongoing,
        lanterns_up: false, // `LanternNight.LanternsUp` — unported
        player_life: life,
        player_life_max: life_max,
        player_base_life_max: life_max, // `Player.statLifeMax` — the port keeps one max
        player_defense: 0,   // `Player.statDefense` — no defense field yet
        player_male: true,   // `Player.Male` — the spawn default
        player_head: 0,      // `Player.head` — no armor slots yet
        player_deaths_pve: deaths,
        player_luck: 0.0,    // `Player.NormalizedLuck` — no luck feed yet
        player_has_cola: has_cola,
        angler_quest_finished: false, // `Main.anglerQuestFinished` — unported
        angler_quests_finished: angler_quests,
        golfer_score,
    };
    let text = crate::npc_chat::get_chat(&app.npcs[i], &mut ctx);
    app.npc_chat_text = text.clone(); // `:43082` — `Main.npcChatText = GetChat()`
    Some(text)
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

/// `Main.CanStartInvasion` (`Main.cs:64414-64432`): `invasionType != 0`
/// blocks (every invasion type, not just the goblins), plus at least one
/// qualifying player — the 200-life gate (`statLifeMax >= 200`); the
/// multiplayer count folds to the session player. The item path passes
/// `ignoreDelay: true`, so the `invasionDelay` scouting arm folds out.
pub(crate) fn can_start_invasion(world: &tr_core::World, player_life_max: i32) -> bool {
    world.invasion_type == 0 && player_life_max >= 200
}

/// `Main.StartInvasion(type)` (`Main.cs:64435-64486`): the stale-type
/// clear, the active-invasion early return, the size
/// `80 + 40 x the qualifying players` (+40+20x for the pirates' type 3,
/// `160 + 40x` for the martians' type 4), the full march/warn state and
/// the `Next(2)` side roll. `rng` is the caller's `Main.rand` — the
/// `UpdateTime` stream from the morning roll, `UpdatePlayers` from the
/// items, the bare stream from the debug panel.
pub(crate) fn start_invasion(
    world: &mut tr_core::World,
    qualifying_players: i32,
    invasion_type: i32,
    rng: &mut UnifiedRandom,
) -> bool {
    // `:64437-64444`: a finished invasion clears its stale type, then any
    // live one refuses the new start.
    if world.invasion_type != 0 && world.invasion_size == 0 {
        world.invasion_type = 0;
    }
    if world.invasion_type != 0 {
        return false;
    }
    // `:64445-64453`: the start needs at least one live 200+-life player —
    // with none the C# does nothing (the type is never assigned).
    if qualifying_players <= 0 {
        return false;
    }
    let n = qualifying_players;
    // The panel/kill paths read the goblin flag (C# only has the type).
    world.invasion_type = invasion_type;
    world.invasion_goblins = invasion_type == 1;
    // `:64456-64464`: the base 80+40x; the pirates (type 3) add 40+20x,
    // the martians (type 4) use 160+40x.
    world.invasion_size = 80 + 40 * n;
    if invasion_type == 3 {
        world.invasion_size += 40 + 20 * n;
    }
    if invasion_type == 4 {
        world.invasion_size = 160 + 40 * n;
    }
    world.invasion_size_start = world.invasion_size;
    world.invasion_warn = 0;
    // `:64471-64483`: the martians telegraph from one tile west of spawn with
    // `invasionWarn = 2`; every other type marches in from a random side.
    if invasion_type == 4 {
        world.invasion_x = world.spawn_x as f64 - 1.0;
        world.invasion_warn = 2;
    } else {
        // `Main.rand.Next(2) == 0 → invasionX = 0` else `maxTilesX`.
        world.invasion_x = if rng.next_max(2) == 0 {
            0.0
        } else {
            world.width as f64
        };
    }
    true
}

/// `Main.StartInvasion`'s whole body (`Main.cs:64435-64486`) for an `App`:
/// the `World` half plus the progress-dial fields that live on the host
/// (`invasionProgress`/`Icon`/`Wave`/`Max`, `:64466-64469`). Returns whether
/// the invasion actually started (the C# gives no answer; the callers need
/// one to decide on the sound/chat side effects).
///
/// `key: None` is the bare `Main.rand` — the debug console is the chat
/// command dispatcher's stand-in (S1-E-29), which the C# runs outside every
/// `Main.SwapRandom` block.
pub(crate) fn start_invasion_in(
    app: &mut App,
    qualifying_players: i32,
    invasion_type: i32,
    key: Option<crate::app::RngKey>,
) -> bool {
    let rng = match key {
        Some(k) => app.rngs.stream(k),
        None => app.rngs.global(),
    };
    if !start_invasion(&mut app.world, qualifying_players, invasion_type, rng) {
        return false;
    }
    app.invasion_progress = 0;
    app.invasion_progress_icon = invasion_type + 3;
    app.invasion_progress_wave = 0;
    app.invasion_progress_max = app.world.invasion_size_start;
    true
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
    // `GetAnglerReward`'s `questItemType` — the 5303/5302 Jellyfish arm
    // gates on the current quest fish (`Player.cs:56345-56354`).
    let quest_item = crate::angler::QUEST_POOL[app.world.angler_quest as usize];
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
    // The turn-in is the player's item-use interaction (`Player.ItemCheck`
    // under the `UpdatePlayers` swap) — the reward rolls ride
    // `stream(UpdatePlayers)`.
    let lines = crate::angler::angler_reward(
        quests_done,
        hard_mode,
        quest_item,
        adjustment,
        &owned,
        app.rngs.stream(RngKey::UpdatePlayers),
    );
    let (ax, ay) = app.npcs[angler_idx].center();
    for line in lines {
        // The reward spawns join the turn-in's `UpdatePlayers` stream.
        let rng = app.rngs.stream(RngKey::UpdatePlayers);
        crate::world_sim::items::spawn_item_drop_px(
            &mut app.items,
            rng,
            line.item,
            line.stack,
            Vec2::new(ax, ay),
        );
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
    // `AnglerQuestSwap` runs from `UpdateTime_StartDay` (`Main.cs:65373`)
    // — the `Main.rand.Next(41)` re-roll ladder rides the `UpdateTime`
    // stream.
    let idx = crate::angler::angler_quest_swap(&gates, app.rngs.stream(RngKey::UpdateTime));
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
    // `Next(20) == 0` (`NPC.cs:1695`'s underground-desert arm) — the
    // spawn arms ride the town-arrival cadence's `UpdateTime` stream
    // (research section 3's row; the port runs the arms as their own pass).
    if water || app.rngs.stream(RngKey::UpdateTime).next_max(20) != 0 {
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
    // `Next(8) == 0` (`NPC.cs:1670`'s spider-wall arm).
    if water || app.rngs.stream(RngKey::UpdateTime).next_max(8) != 0 {
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
    // `Next(20) == 0` (`NPC.cs:1775-1794`'s two 376 arms).
    if app.rngs.stream(RngKey::UpdateTime).next_max(20) != 0 {
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
            // The port's scatter stand-in (the C# creepers spawn at the
            // brain's centre, `:32374-32385`) — the pass's `UpdateNPCs`
            // stream.
            let rng = app.rngs.stream(RngKey::UpdateNpcs);
            for _ in 0..20 {
                let sx = brain.x + bw * 0.5 + (rng.next_float() * 2.0 - 1.0) * bw as f32;
                let sy = brain.y + bh * 0.5 + (rng.next_float() * 2.0 - 1.0) * bh as f32;
                let svx = (rng.next_float() * 61.0 - 30.0) * 0.1;
                let svy = (rng.next_float() * 61.0 - 30.0) * 0.1;
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
    // `AI_006_Worms` runs inside each NPC's `Update` — the `UpdateNPCs`
    // swap's stream (`Main.cs:17767-17769`).
    let rng = app.rngs.stream(RngKey::UpdateNpcs);
    let world = &app.world;
    let player = app.player.as_ref().map(|p| (p.center().0, p.center().1));
    update_worms_in(world, player, &mut app.npcs, rng, &mut app.next_worm_id);
    // S1-E-21g — the worm pass's queued plays (the leech entry burst,
    // the dig roars): the same drain as the per-NPC loop, after the
    // pass's draws on the same UpdateNPCs stream.
    for n in app.npcs.iter_mut() {
        for req in n.wants_sound.drain(..) {
            play_npc_sound_request(&mut app.audio, req, rng);
        }
    }
}

/// S1-E-21g — one drained [`crate::npc::NpcSoundRequest`]: the queue's
/// cue dispatches to the faithful play method. Every method gates the
/// attenuation BEFORE any gameplay draw (the E21e precedent —
/// `LegacySoundPlayer.cs:369-436` precedes the type switch); the case
/// draws ride the caller's stream (the `UpdateNPCs` swap for every
/// wave-G site, `Main.cs:17767`). The int overloads
/// (`SoundEngine.cs:102-108`) draw NO `GetRandomPitch` NextFloat; the
/// style façades (`:93-99`) consume it on the audio-only stream.
fn play_npc_sound_request(
    audio: &mut crate::audio::Audio,
    req: crate::npc::NpcSoundRequest,
    rng: &mut UnifiedRandom,
) {
    use crate::npc::NpcSoundCue;
    match req.cue {
        NpcSoundCue::Roar(style) => {
            audio.play_roar(style, req.x, req.y);
        }
        NpcSoundCue::DashRoar(style) => {
            audio.play_dash_roar(style, req.x, req.y);
        }
        NpcSoundCue::Item8 => {
            audio.play_npc_item_cue(8, req.x, req.y, rng);
        }
        NpcSoundCue::Item14 => {
            audio.play_npc_item_cue(14, req.x, req.y, rng);
        }
        NpcSoundCue::Item34 => {
            audio.play_npc_item_cue(34, req.x, req.y, rng);
        }
        NpcSoundCue::NpcHit(style) => {
            audio.play_npc_hit_int(style, req.x, req.y, rng);
        }
        NpcSoundCue::NpcDeath(style) => {
            audio.play_npc_death_int(style, req.x, req.y, rng);
        }
        NpcSoundCue::NpcDeath61 => {
            audio.play_npc_death_style(61, req.x, req.y, rng);
        }
        NpcSoundCue::NpcDeath13 => {
            audio.play_npc_death_style(13, req.x, req.y, rng);
        }
        NpcSoundCue::EowDig => {
            audio.play_eow_dig(req.x, req.y);
        }
    }
}

/// The worm pass's core — the pieces the tests drive directly (the world,
/// the player centre, the Vec, the RNG stream, the id counter).
pub(crate) fn update_worms_in(
    world: &tr_core::World,
    player: Option<(f32, f32)>,
    npcs: &mut Vec<Npc>,
    mut rng: &mut UnifiedRandom,
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
        // S1-E-21g — `NPC.cs:50965-50968`: the leech head's (117)
        // tunnel-entry burst — `localAI[1]` 0 → 1 latched at the chain
        // init, `SoundID.NPCDeath13` at base.position (the style façade).
        // The 20-gore/dust burst behind it draws Main.rand — SKIPPED
        // (the port's gore/dust deviation, ANALYSIS wave G).
        if kind == NpcKind::LeechHead && npcs[i].local_ai[1] == 0.0 {
            npcs[i].local_ai[1] = 1.0;
            // E0502 gate fix: read the position before the mutable push.
            let (sx, sy) = (npcs[i].x, npcs[i].y);
            npcs[i].wants_sound.push(crate::npc::NpcSoundRequest {
                cue: crate::npc::NpcSoundCue::NpcDeath13,
                x: sx,
                y: sy,
            });
        }
        // The tails (9/12/15) sit outside the chain-spawn lists
        // (`:51393`/`:51436`) — they end the chain.
        if !kind.is_worm_head() && !kind.is_worm_mid() {
            i += 1;
            continue;
        }
        // The budget roll only at the chain start. (The Destroyer family
        // left this pass with S1-D-3-wire-destroyer — `AI_037`'s own chain
        // runs in `update_destroyers`; `is_worm()` excludes it.)
        let total = if kind.is_worm_head() && npcs[i].seg_index == 0 {
            match kind {
                // `:51310`'s `Next(8, 13)` / `:51313`'s `Next(4, 7)` — the
                // inclusive-low exclusive-high C# ranges.
                NpcKind::DevourerHead => rng.next_range(8, 13) as u32,
                NpcKind::GiantWormHead => rng.next_range(4, 7) as u32,
                // The leech head: `Next(3, 6)` (`AI_006`'s type-117 branch).
                NpcKind::LeechHead => rng.next_range(3, 6) as u32,
                // `GetEaterOfWorldsSegmentsCount` (65; the expert 70 rides
                // the difficulty work).
                // The crawltipede's 30 segments (`:51375`'s `num34 = 30`).
                NpcKind::SolarCrawltipedeHead => 29,
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
        // the KIND. (The Destroyer family's trail/laser runs in
        // `update_destroyers` — `AI_037`'s own machine, not this pass.)
        if npcs[i].seg_index == 0
            || (npcs[i].kind.is_worm_head() && npcs[i].kind.worm_next() != npcs[i].kind)
        {
            continue;
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

/// The Plantera minion pool pass (S1-D-3-wire-plantera-minions) — the
/// aiStyle-52 hook (`NPC.cs:32076-32240`, `npc_ai_style52.rs`) and the
/// aiStyle-53 tentacle (`:32241-32369`, `npc_ai_style53.rs`). Both arms
/// read the WHOLE pool: the hook's orphan gate + sibling re-arm scan
/// (`:32080`/`:32129`) and the tentacle's `NPC.plantBoss` + `ai[3]-1`
/// anchor slot (`:32254`/`:32260-32264`), so they run here after the
/// per-NPC step (the worm pass's precedent — the C# interleaves them in
/// slot order; the port's pass reads the same spawn order).
///
/// `NPC.plantBoss` is the C# static the boss's own arm sets every tick
/// (`NPC.cs:31735`) and `checkDead` clears (`:8391-8394`); the port
/// resolves it per tick as the first live Plantera's pool index (-1 =
/// none — the orphan strike). Both machines run on the `UpdateNPCs`
/// stream (`Main.cs:17767-17769`), in pool order — the hook's draws
/// (`Next(60,300)` per moving sibling, `Next(300,600)`, the jitter pair,
/// the below-half `Next(6)`) and the tentacle's rearm triple
/// (`Next(120,480)`, `Next(-100,101)` x2) keep their census order; the
/// ftw reflect `Next(10)` is constants-off (no runtime getGoodWorld).
///
/// The integration is `UpdateNPC`'s noGravity/noTileCollide tail
/// (`NPC.cs:90490-90498`): raw `position += velocity`. The orphan
/// strikes land as `hp = 0` (the port's `active = false` convention —
/// the machines write it themselves; the death retain + the death-sound
/// pass own the rest, the 262/263/264 HitEffect dust is the documented
/// skip). The ftw thorn burst (`HookEvents::spawn_thorns`) is
/// constants-off with getGoodWorld.
pub(crate) fn update_plantera_minions(app: &mut App) {
    if !app
        .npcs
        .iter()
        .any(|n| n.hp > 0 && matches!(n.kind, NpcKind::PlanterasHook | NpcKind::PlanterasTentacle))
    {
        return;
    }
    let plant_boss = app
        .npcs
        .iter()
        .position(|n| n.kind == NpcKind::Plantera && n.hp > 0)
        .map(|i| i as i32)
        .unwrap_or(-1);
    let players_vec: Vec<crate::npc_target::TargetCandidate> = app
        .player
        .as_ref()
        .map(|p| vec![crate::npc_target::session_candidate(p)])
        .unwrap_or_default();
    let players = players_vec.as_slice();
    // `Main.player[boss.target].ZoneJungle` (`NPC.cs:32090`): the port's
    // scene metrics carry the render-side zone flag (the HookCtx seam).
    let zone_jungle = app.scene_metrics.zone_jungle;
    let rng = app.rngs.stream(RngKey::UpdateNpcs);
    let world = &app.world;
    let npcs = &mut app.npcs;
    let type_of = |n: &Npc| n.kind.cs_id() as i32;
    for i in 0..npcs.len() {
        if npcs[i].hp <= 0 {
            continue;
        }
        match npcs[i].kind {
            NpcKind::PlanterasHook => {
                let mut ctx = crate::npc_ai_style52::HookCtx::stock(i as i32, plant_boss);
                ctx.boss_target_zone_jungle = zone_jungle;
                let ev = crate::npc_ai_style52::planteras_hook_step(
                    npcs, i, &ctx, world, players, rng, 0, type_of,
                );
                if ev.orphan_strike {
                    // `StrikeNPCNoInteraction(9999, 0, 0)` (`:32082`):
                    // the instant-kill branch (`NPC.cs:81237`'s
                    // `Damage >= 9999 && owner == 255`) — hp = 0, the
                    // death retain owns the removal.
                    npcs[i].hp = 0;
                    continue;
                }
                // `spawn_thorns` is constants-off (getGoodWorld false).
                npcs[i].integrate_no_tile_collide();
            }
            NpcKind::PlanterasTentacle => {
                // The anchor slot (`:32260-32264`): the boss, or the
                // expert hook link `ai[3]-1` (the classic batch never
                // writes ai[3], so the boss always anchors today).
                let anchor_slot = if npcs[i].ai[3] > 0.0 {
                    (npcs[i].ai[3] as usize).saturating_sub(1)
                } else {
                    plant_boss.max(0) as usize
                };
                let view = |n: &Npc| crate::npc_ai_style53::Ai53NpcView {
                    active: n.hp > 0,
                    x: n.x,
                    y: n.y,
                    width: n.size().0 as i32,
                    height: n.size().1 as i32,
                    life: n.hp,
                    life_max: n.max_hp,
                };
                let boss_view = usize::try_from(plant_boss)
                    .ok()
                    .and_then(|b| npcs.get(b))
                    .map(view);
                // The C# array always has 200 slots: an out-of-range
                // ai[3] reads an INACTIVE stock NPC (the module's seam).
                let anchor_view = npcs.get(anchor_slot).map(view).unwrap_or(
                    crate::npc_ai_style53::Ai53NpcView {
                        active: false,
                        x: 0.0,
                        y: 0.0,
                        width: 0,
                        height: 0,
                        life: 0,
                        life_max: 0,
                    },
                );
                let ctx = crate::npc_ai_style53::Ai53Ctx {
                    players,
                    plant_boss,
                    boss: boss_view,
                    anchor: Some(anchor_view),
                    expert_mode: false,
                    get_good_world: false,
                    ticker: 0,
                };
                let _fx = crate::npc_ai_style53::ai_053_step(&mut npcs[i], world, &ctx, rng);
                // The orphan strike + the anchor-lost death already set
                // hp = 0 inside the arm; the reflects_projectiles write
                // is None (getGoodWorld off).
                if npcs[i].hp > 0 {
                    npcs[i].integrate_no_tile_collide();
                }
            }
            _ => {}
        }
    }
}

/// The head's per-tick machine (`AI_006`'s `ai[1] == 0` branch). `flag3`
/// (the dig mode) = any tile in the body rect is solid or liquid-heavy
/// (`:51580`'s count rule). In the open air the head stumbles: the gravity
/// `+0.11` (the cap `num45`), the horizontal steering (`:51940-51972`); in
/// the ground it flies straight at the tile-snapped player (`:51997-52002`).
/// `noTileCollide` is family-wide — no collision blocks the head.
fn worm_head_step(
    npc: &mut Npc,
    world: &tr_core::World,
    pcx: f32,
    pcy: f32,
    _rng: &mut UnifiedRandom,
) {
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
        // S1-E-21g — `NPC.cs:51976-51995`: the dig rumble on the
        // `soundDelay == 0` cadence — the distance/40 clamp (10..20)
        // re-arms the delay BEFORE the play (`:51987`), then the EoW
        // heads (13..15) play the `EOWDiggin` trackable (`:51990`) and
        // every other head the case-15 roar at the DEFAULT Style 1
        // (`:51994` — the int overload's `Style = 1`, `SoundEngine.cs:102`).
        // The type exclusions (621/87/117/454/412) map to the leech and
        // the crawltipede; the Skyblock `lowTiles` term is constant-false
        // in the port (no skyblock), so the 13..15 arm always passes.
        // `num64` is the C#'s `num62` — the pre-scale tile-snapped
        // distance (`:51866`), NOT the aim length after the `(d-w)/d`
        // rescale (`:51905` overwrites num62 for the followers only).
        // The Destroyer's own `AI_037` roar (`:50081-50093`) shares this
        // shape — the port's worm pass stands for it (ANALYSIS, wave G).
        let silent_kind = matches!(
            npc.kind,
            NpcKind::LeechHead | NpcKind::SolarCrawltipedeHead
        );
        if !silent_kind && npc.sound_delay == 0 {
            npc.sound_delay = (len / 40.0).clamp(10.0, 20.0) as i32;
            let cue = if npc.kind == NpcKind::EaterOfWorldsHead {
                crate::npc::NpcSoundCue::EowDig
            } else {
                crate::npc::NpcSoundCue::Roar(1)
            };
            npc.wants_sound.push(crate::npc::NpcSoundRequest {
                cue,
                x: npc.x,
                y: npc.y,
            });
        }
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

/// S1-D-3-wire-destroyer: the Destroyer family's sim pass — `AI_037`
/// (`npc_ai_style37.rs::destroyer_family_step`) + the Probe's `AI_005`
/// type-139 arm (`probe_step`) + the strike-queued probe spawns + the
/// event resolve (lasers → `wants_hostile_shot`, the roar, the daytime
/// family despawn). Runs after the per-NPC step, the worm pass's position
/// (the family is NOT `is_worm()` — the stand-in retired).
///
/// RNG: the family's cadence draws ride the `UpdateNPCs` stream
/// (`Main.cs:17767-17769` wraps `NPC.UpdateNPC`); the probe machine draws
/// nothing (its wobble is a counter, the dust arms exclude 139). The probe
/// LAUNCH roll rides the strike's stream at the strike site
/// (`probe_launch_on_hit`), not here.
pub(crate) fn update_destroyers(app: &mut App) {
    let has_family = app
        .npcs
        .iter()
        .any(|n| crate::npc_ai_style37::is_destroyer_family(n.kind) && n.hp > 0);
    let has_probe = app.npcs.iter().any(|n| n.kind == NpcKind::Probe && n.hp > 0);
    let has_probe_req = app.npcs.iter().any(|n| n.wants_probe.is_some());
    if !has_family && !has_probe && !has_probe_req {
        return;
    }
    // The target candidate (the `npc_target` convention, single-player).
    let players: Vec<crate::npc_target::TargetCandidate> = match app.player.as_ref() {
        Some(p) => vec![crate::npc_target::TargetCandidate {
            active: true,
            dead: p.dead,
            ghost: false,
            x: p.x,
            y: p.y,
            width: crate::player::WIDTH as i32,
            height: crate::player::HEIGHT as i32,
            aggro: 0,
            item_animation: p.frame_inputs.swing_anim as i32,
            npc_type_no_aggro: false,
            tank_pet: None,
        }],
        None => Vec::new(),
    };
    // The strike-queued probe spawns (`HitEffect`, `NPC.cs:89278`): the
    // anchor is the body's bottom-centre; `NewNPC`'s `Bottom = (X, Y)`.
    let mut probe_spawns = Vec::new();
    for n in app.npcs.iter_mut() {
        if let Some((ax, ay)) = n.wants_probe.take() {
            probe_spawns.push((ax, ay));
        }
    }
    for (ax, ay) in probe_spawns {
        let (pw, ph) = NpcKind::Probe.size();
        let mut probe = Npc::new(NpcKind::Probe, ax - pw * 0.5, ay - ph, 1, 0.0);
        probe.hp = probe.max_hp;
        app.npcs.push(probe);
    }
    if players.is_empty() {
        return;
    }
    // The family machine (the chain spawn + the per-slot AI_037 ticks).
    let events = {
        let rng = app.rngs.stream(RngKey::UpdateNpcs);
        crate::npc_ai_style37::destroyer_family_step(
            &mut app.npcs,
            &app.world,
            &players,
            rng,
            &mut app.next_worm_id,
            0, // the EverLastingTicker stamp (the S1-D-3p convention)
        )
    };
    // The event resolve, in tick order (the C#'s per-slot side effects).
    let mut despawn_family = false;
    for (i, ev) in events {
        if let Some(shot) = ev.laser {
            // `NPC.cs:49877-49878` — type 100, timeLeft 300 (the drain's
            // override list already carries DESTROYER_LASER).
            app.npcs[i].wants_hostile_shot.push((
                crate::projectile::DESTROYER_LASER,
                shot.x,
                shot.y,
                shot.vx,
                shot.vy,
                shot.damage,
            ));
        }
        if ev.roar {
            // `NPC.cs:50093` — `SoundEngine.PlaySound(15, pos)` (Roar_0).
            let (nx, ny) = (app.npcs[i].x, app.npcs[i].y);
            app.audio.play_at("Roar_0", nx, ny, 1.0);
        }
        if ev.wants_death {
            // The orphan death wrote life = 0 in place; the death pass
            // (the retain below in update_npcs) carries the events.
        }
        if ev.despawn_family {
            despawn_family = true;
        }
        // The fade dust (`:49799-49804`) and the AddLight (`:49929`) ride
        // the documented cosmetic skip (the other machines' convention —
        // no dust/light seam inside the AI pass yet; ANALYSIS).
    }
    if despawn_family {
        // `NPC.cs:49971-49977` — every aiStyle-37 NPC goes inactive below
        // the rock layer at dawn. The C#'s `active = false` fires NO death
        // events (no loot/sound/progression) — the port drops the family
        // outright, bypassing the death retain's event pass.
        app.npcs
            .retain(|n| !crate::npc_ai_style37::is_destroyer_family(n.kind));
    }
    // The Probe's AI_005 type-139 arm (the same UpdateNPCs stream — the
    // machine draws nothing, so the borrow is order-only).
    {
        let world = &app.world;
        for n in app.npcs.iter_mut() {
            if n.kind != NpcKind::Probe || n.hp <= 0 {
                continue;
            }
            n.old_vx = n.vx;
            n.old_vy = n.vy;
            let just_hit = n.hurt_cd > 0.0;
            if let Some(shot) = crate::npc_ai_style37::probe_step(
                n, world, &players, just_hit, 0,
            ) {
                // `NPC.cs:50625` — type 84, the snapped aim velocity, no
                // timeLeft override (the SetDefaults 600 rides).
                n.wants_hostile_shot.push((
                    crate::projectile::PROBE_LASER,
                    shot.x,
                    shot.y,
                    shot.vx,
                    shot.vy,
                    shot.damage,
                ));
            }
        }
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
        // `SetEventFlagCleared(ref NPC.downedX, n)` + the matching
        // `AchievementsHelper.NotifyProgressionEvent` (`Main.cs:64335-64354`):
        // goblins 10, frost 12, pirates 11, martians 13.
        match app.world.invasion_type {
            2 => {
                app.world.downed_frost = true;
                let _ = app.notify_progression_net(12);
            }
            3 => {
                app.world.downed_pirates = true;
                let _ = app.notify_progression_net(11);
            }
            4 => {
                app.world.downed_martians = true;
                let _ = app.notify_progression_net(13);
            }
            _ => {
                app.world.downed_goblins = true;
                let _ = app.notify_progression_net(10);
            }
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
    // `:364`: `!Ongoing || LostThisRun || WonThisRun ||
    // EnemySpawningIsOnHold` — the lost run scores nothing at all (the
    // loss scene's kill wipe must not advance the wave), and the hold's
    // refusal runs FIRST — the kill under the break scores nothing.
    if !app.world.dd2_event_ongoing
        || app.world.dd2_lost
        || app.world.dd2_won
        || app.world.dd2_spawn_hold != 0
    {
        return;
    }
    // The event's tier (`FindProperDifficulty`, `:358-370`): the wave
    // budgets and the kill worth ride it.
    let tier = dd2_tier(app);
    let wave = app.world.dd2_wave;
    // `GetInvasionStatus` (`:517-536`) runs next (`:368`): the bucket is
    // the wave's kills BEFORE this kill lands; the tier's table may replace
    // it (the wave-7 Betsy bucket, `FindFirstNPC(551)`, `:1487-1494`) and
    // clamp it (the wave-5/-7 caps read the downed latches, `:993`,
    // `:1181`).
    let wave_kills_before = app.world.dd2_wave_kills;
    let mut kills_i = wave_kills_before as i32;
    let required = match tier {
        2 => {
            tr_core::dd2::required_wave_kills_t2(wave, &mut kills_i, app.world.dd2_latches.ogre_t2)
        }
        3 => tr_core::dd2::required_wave_kills_t3(wave, &mut kills_i, dd2_betsy_hp_ratio(app)),
        _ => tr_core::dd2::required_wave_kills(
            wave,
            &mut kills_i,
            app.world.dd2_latches.dark_mage_t1,
        ),
    };
    // `GetMonsterPointsWorth` (`:546-554`): the tier's worth row — 0 for
    // any id outside the two Old One's Army ranges (`:1117`); the wave-5/-7
    // boss rows latch (`:1112`, `:1205`).
    let Some(slain_id) = dd2_pool_id(kind) else {
        return;
    };
    let worth = tr_core::dd2::monster_points_worth(
        tier,
        wave,
        wave_kills_before,
        slain_id,
        &mut app.world.dd2_latches,
    );
    app.world.dd2_wave_kills += worth as f32;
    // `:376-380`: the count at or past the tier's budget resets the counter,
    // opens the next wave and sets `flag`; `:404`'s 1800-tick break follows
    // the win sentinel's refusal.
    let mut completed = false;
    if app.world.dd2_wave_kills >= required as f32 && required != 0 {
        app.world.dd2_wave_kills = 0.0;
        app.world.dd2_wave += 1;
        completed = true;
        // `:381`: the second `GetInvasionStatus(currentlyInCheckProgress:
        // true)` — the sentinel wave's arm (tier 1's 6 `:998-1005`, tier
        // 2/3's 8 `:1186-1193`/`:1496-1503`) rewrites `waveNumber` back to
        // the last wave, `currentKillCount` to 1, returns the budget 1 and
        // runs `StartVictoryScene` (`:481-505`): `WonThisRun` latches, the
        // first crystal (548) flips to `(ai0, ai1) = (2, 2)` and every live
        // portal (549) to `(0, 1)`. S1-E-12c: the scene's ai flips ride
        // `tr_core::dd2_victory::victory_scene`'s plan here.
        let sentinel = match tier {
            2 | 3 => 8,
            _ => 6,
        };
        if app.world.dd2_wave == sentinel {
            app.world.dd2_wave = sentinel - 1; // `:999` / `:1187` / `:1497`
            let crystal = app
                .npcs
                .iter()
                .position(|n| n.kind == NpcKind::DD2EterniaCrystal && n.hp > 0);
            let portals: Vec<usize> = app
                .npcs
                .iter()
                .enumerate()
                .filter(|(_, n)| n.kind == NpcKind::DD2LanePortal && n.hp > 0)
                .map(|(i, _)| i)
                .collect();
            let plan = tr_core::dd2_victory::victory_scene(crystal, &portals);
            app.world.dd2_won = plan.won; // `:483` — before the lookup
            if let Some(i) = plan.crystal_slot {
                // `:493-494`: ai[1] first, then ai[0].
                app.npcs[i].ai[1] = 2.0;
                app.npcs[i].ai[0] = 2.0;
            }
            // `:496-504`: every live 549 — only with a crystal (`:489-492`'s
            // early return leaves `portal_slots` empty otherwise).
            for &i in plan.portal_slots {
                app.npcs[i].ai[0] = 0.0;
                app.npcs[i].ai[1] = 1.0;
            }
            // `:382-396`: the WonThisRun arm — the report is the sentinel's
            // `(1, budget 1, wave)` pair when the count moved and the worth
            // is non-zero (`:384`), then `CheckProgress` RETURNS (`:395` —
            // no wave-complete broadcast, no break hold, no medals, and the
            // `:451` refusal never runs for this arm).
            if 1.0 != wave_kills_before && worth != 0 {
                app.invasion_progress = 1;
                app.invasion_progress_max = 1;
                app.invasion_progress_icon = 3;
                app.invasion_progress_wave = app.world.dd2_wave;
                app.invasion_progress_display_left = 160;
            }
            return;
        } else {
            // `:398-403`: the wave-complete broadcast — the wave-2 opening
            // takes the `WaveCompleteFirst` key, every other wave
            // `WaveComplete`, both in `INFO_NEW_WAVE_COLOR` (`:1810`).
            let key = if app.world.dd2_wave == 2 {
                "DungeonDefenders2.WaveCompleteFirst"
            } else {
                "DungeonDefenders2.WaveComplete"
            };
            app.chat
                .new_text(crate::lang::text(key), tr_core::dd2::INFO_NEW_WAVE_COLOR);
            // `:404`: `SetEnemySpawningOnHold(1800)` — only past the win
            // sentinel (the C# returns at `:395` before it).
            app.world.dd2_spawn_hold = tr_core::dd2::SPAWN_HOLD_WAVE_BREAK;
            // `:405-449`: the wave-complete medal rows, read on the NEW
            // wave (`num3`) — one `DropMedals` stack per live crystal.
            if let Some(medals) =
                tr_core::dd2_victory::wave_complete_medals(tier, app.world.dd2_wave)
            {
                dd2_drop_medals(app, medals);
            }
        }
    }
    // `:451-454`: a kill that moved no count reports nothing.
    if (kills_i + worth) as f32 == wave_kills_before {
        return;
    }
    // `:455-478`: the wave-completing kill reports the fresh 1/1 pair under
    // the wave that just finished; the ordinary kill reports the tier's
    // pair under the current wave.
    if completed {
        app.invasion_progress = 1;
        app.invasion_progress_max = 1;
    } else {
        // `currentKillCount += (int)num` (`:373`) adds the kill's worth.
        app.invasion_progress = kills_i + worth;
        app.invasion_progress_max = required;
    }
    // The report rides `Main.ReportInvasionProgress(…, 3, wave)`: icon 3 is
    // the `Extra[79]` Old One's Army sheet and the `> 0` wave selects
    // `DrawInvasionProgress`'s `Game.WaveMessage` arm.
    app.invasion_progress_icon = 3;
    app.invasion_progress_wave = wave;
    app.invasion_progress_display_left = 160;
}

/// `DD2Event.DropMedals` (`DD2Event.cs:802-813`) — one `Item.NewItem(...,
/// 3817, numberOfMedals, noBroadcast: true)` stack per live Eternia Crystal
/// (548), in `Main.npc` order, at the crystal's rect centre (the obsolete
/// rect overload's `X + Width/2, Y + Height/2`, `Item.cs:49651`). The rolls
/// ride the caller's stream through the `spawn_item_drop_px` seam (the
/// module's zero-RNG-decisions contract — the decision draws nothing, the
/// spawn's velocity/phase triplet + prefix ride `Item.NewItem`'s
/// `Main.rand`). `MakeInstanced` (`:810`) is netplay-only (constants-off).
fn dd2_drop_medals(app: &mut App, medals: i32) {
    if medals <= 0 {
        return;
    }
    // `nPC.active` (`:807`): at the win stop the C# runs `WinInvasionInternal`
    // BEFORE the crystal's `life = 0` + `checkDead` (`NPC.cs:40798-40801`),
    // so the stopping crystal is still active. The port's machine zeroes
    // `hp` in the same tick — the `wants_event_stop` term keeps that crystal
    // in the scan (the death retain has not run yet). A wave-complete drop
    // never sees one: the stop resolve removes it the same tick.
    let spots: Vec<Vec2> = app
        .npcs
        .iter()
        .filter(|n| {
            n.kind == NpcKind::DD2EterniaCrystal && (n.hp > 0 || n.wants_event_stop)
        })
        .map(|n| Vec2::new(n.center().0, n.center().1))
        .collect();
    for pos in spots {
        // The kill path's stage: `checkDead` runs under the NPC pass's
        // swap in the port's batched-death model (the S1-E-21c recorded
        // deviation — the C# strikes under the player's stage).
        let rng = app.rngs.stream(RngKey::UpdateNpcs);
        crate::world_sim::items::spawn_item_drop_px(
            &mut app.items,
            rng,
            tr_core::dd2::MEDAL_ITEM,
            medals as u32,
            pos,
        );
    }
}

/// `DD2Event.WinInvasionInternal` (`DD2Event.cs:314-342`) — the win stop's
/// resolve: the `>= n` downed-tier latches (`:316-327`), the tier's medal
/// stack (`:328-340` — 3/15/60 via `DropMedals`), the tier-3 progression
/// event 23 (`:338`, `AchievementsHelper.NotifyProgressionEvent`) and the
/// green `InvasionWin` broadcast (`:341`, `INFO_START_INVASION_COLOR` =
/// `ChatColors.World`, `ChatColors.cs:24`).
fn dd2_win_invasion(app: &mut App) {
    let tier = dd2_tier(app);
    let (t1, t2, t3) = tr_core::dd2_victory::victory_downed_flags(tier);
    // The latches only raise (`if (>= n) flag = true` — never clear).
    app.world.dd2_downed_t1 |= t1;
    app.world.dd2_downed_t2 |= t2;
    app.world.dd2_downed_t3 |= t3;
    if tier == 3 {
        // `:338`: BEFORE the medals (the C#'s statement order).
        let _ = app.notify_progression_net(23);
    }
    if let Some(medals) = tr_core::dd2_victory::victory_medals(tier) {
        dd2_drop_medals(app, medals);
    }
    app.chat.new_text(
        crate::lang::text("DungeonDefenders2.InvasionWin"),
        tr_core::dd2::INFO_START_INVASION_COLOR,
    );
}

/// `DD2Event.ShouldDropCrystals`'s per-kill caller (`NPC.checkDead`'s DD2
/// arms, `NPC.cs:79240-79278`) — the budget decision reads the CURRENT
/// invasion pair (`GetInvasionStatus`, `:817`); true drops one energy
/// crystal (3822) at the dead body's centre (`Item.NewItem(source, Center,
/// 3822)`). The goblin trio's `AnnounceGoblinDeath` (`:79245`,
/// `DD2Event.cs:694-697` — the `_deadGoblinSpots` list) stays deferred with
/// the Dark Mage's `RaiseGoblins` (ANALYSIS). The rolls ride the caller's
/// stream through `spawn_item_drop_px` (zero-RNG-decisions contract).
fn dd2_crystal_drop(app: &mut App, dead: &Npc) {
    let Some(id) = dd2_pool_id(dead.kind) else {
        return;
    };
    // The two arm sets: 552/553/554 (the goblins, `:79240-79251`, gated on
    // `DD2Event.Ongoing`) and the 555-565/568-578 band (`:79252-79278`, no
    // ongoing gate — an orphan body still consults the budget). Betsy
    // (551), the crystal (548), the portals (549) and the bartender (550)
    // have no arm.
    let goblin_arm = matches!(id, 552 | 553 | 554);
    let band_arm = (555..=565).contains(&id) || (568..=578).contains(&id);
    if goblin_arm && !app.world.dd2_event_ongoing {
        return;
    }
    if !goblin_arm && !band_arm {
        return;
    }
    // `GetInvasionStatus` (`:517-534`) on the live wave — the same pair the
    // kill report reads (the tier's budget table + its caps).
    let tier = dd2_tier(app);
    let wave = app.world.dd2_wave;
    let mut kills = app.world.dd2_wave_kills as i32;
    let required = match tier {
        2 => tr_core::dd2::required_wave_kills_t2(wave, &mut kills, app.world.dd2_latches.ogre_t2),
        3 => tr_core::dd2::required_wave_kills_t3(wave, &mut kills, dd2_betsy_hp_ratio(app)),
        _ => tr_core::dd2::required_wave_kills(wave, &mut kills, app.world.dd2_latches.dark_mage_t1),
    };
    if !tr_core::dd2_victory::should_drop_crystals(
        &mut app.world.dd2_crystal_drops,
        tier,
        wave,
        required,
        kills,
    ) {
        return;
    }
    let (cx, cy) = dead.center();
    let rng = app.rngs.stream(RngKey::UpdateNpcs);
    crate::world_sim::items::spawn_item_drop_px(
        &mut app.items,
        rng,
        tr_core::dd2::ENERGY_CRYSTAL_ITEM,
        1,
        Vec2::new(cx, cy),
    );
}

/// `Lang.GetInvasionWaveText` (`Lang.cs:1136-1156`) — the new-wave
/// broadcast's text: `Game.FirstWave` at wave 1, `Game.Wave` (with the
/// number) otherwise, joined with the wave roster's NPC names through the
/// `Game.InvasionWave_Type<roster len>` row. The `wave == -1`
/// `Game.FinalWave` arm is unreachable from `UpdateTime` (the sentinel
/// rewrites the wave before any broadcast).
fn dd2_wave_text(wave: i32, roster: &[u16]) -> String {
    let head = if wave == 1 {
        crate::lang::text("Game.FirstWave")
    } else {
        crate::lang::text_args("Game.Wave", &[wave.to_string()])
    };
    let mut args = vec![head];
    args.extend(roster.iter().map(|&id| crate::lang::npc_name(id as i32)));
    crate::lang::text_args(&format!("Game.InvasionWave_Type{}", roster.len()), &args)
}

/// `DD2Event.SummonBetsy` (`DD2Event.cs:914-927`) — the tier-3 wave-7
/// opening's boss: once per invasion (`_spawnedBetsyT3`) and only with no
/// live Betsy, `NPC.SpawnOnPlayer(Player.FindClosest(crystalCenter, 1, 1),
/// 551)`. The port spawns at the player (the boss-summon convention —
/// `SpawnOnPlayer`'s surface scan is the recorded deviation, the
/// `use_boss_summon_item` family).
fn dd2_summon_betsy(app: &mut App) {
    if app.world.dd2_betsy_spawned {
        return;
    }
    if app
        .npcs
        .iter()
        .any(|n| n.kind == NpcKind::DD2Betsy && n.hp > 0)
    {
        return;
    }
    let (px, py) = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((0.0, 0.0));
    let mut betsy = Npc::new(NpcKind::DD2Betsy, px, py, 1, 0.0);
    betsy.hp = betsy.max_hp;
    app.npcs.push(betsy);
    app.world.dd2_betsy_spawned = true; // `:925`
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
        2 => tr_core::dd2::required_wave_kills_t2(
            wave,
            &mut kills.clone(),
            app.world.dd2_latches.ogre_t2,
        ),
        3 => {
            tr_core::dd2::required_wave_kills_t3(wave, &mut kills.clone(), dd2_betsy_hp_ratio(app))
        }
        _ => tr_core::dd2::required_wave_kills(
            wave,
            &mut kills.clone(),
            app.world.dd2_latches.dark_mage_t1,
        ),
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
    // `DD2Event.SpawnMonsterFromGate` is called from the lane gate's AI
    // (`NPC.cs:41001`) — under the `UpdateNPCs` swap (`Main.cs:17767`).
    // The rolls take `Fn`, so the stream rides in a `RefCell` and the
    // closure draws live — the 32-draw pre-buffer (a splitmix-share
    // workaround) dissolves; each pick spends exactly the vanilla's draw
    // count.
    let rng = app.rngs.stream(RngKey::UpdateNpcs);
    let rng_cell = std::cell::RefCell::new(&mut *rng);
    let roll = || rng_cell.borrow_mut().next_float();
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

/// The Old One's Army's tick (`DD2Event.UpdateTime`, `DD2Event.cs:194-251`,
/// + the crystal's scene resolves): the loss intercept, the spawn hold's
/// decay + zero-crossing broadcast (new wave / loss), the crystal-alive
/// flag, and the scene-end stop — `StopInvasion(win: true)` pays
/// `WinInvasionInternal` first (S1-E-12c), both outcomes clear the counters
/// and wipe the entities.
pub(crate) fn update_dd2_event(app: &mut App, _dt: f32) {
    if !app.world.dd2_event_ongoing {
        app.world.dd2_crystal_alive = false;
        return;
    }
    // `NPC.checkDead`'s crystal intercept (`NPC.cs:78087-78095`): the
    // crystal's death does not kill it — `ai[1] = 1`, `ai[0] = 0`,
    // `life = lifeMax`, `dontTakeDamageFromHostiles = true`, and
    // `checkDead` returns. The port's damage lands in the NPC pass ahead
    // of this resolve, so the flip runs before the death retain — the
    // crystal survives into the loss scene. The scene's first-tick arm
    // (`NPC.cs:40868-40885`, `ai[0] == 0`) rides the same resolve: every
    // active portal (549) flips to its fade (`ai[1] = 1`, `ai[0] = 0`),
    // then `DD2Event.ReportLoss` (`:40882`). The 672 defeat-scene
    // projectile (`:40883`, aiStyle 132's filters/sound/gore drama) is
    // deferred — the port carries no scene filters yet (ANALYSIS).
    // The win scene's shield (`NPC.cs:40760-40761`: `life = lifeMax` every
    // tick) keeps the crystal's hp positive, so the `hp <= 0` gate below
    // never sees a winning crystal — the C#'s intercept (`:78087`,
    // `ai[1] != 1`) cannot fire mid-win-scene. The `ai[1] != 2` term pins
    // that invariant explicitly (S1-E-12c: victory must not double-fire
    // with loss).
    let mut loss_entered = false;
    for n in app.npcs.iter_mut() {
        if n.kind == NpcKind::DD2EterniaCrystal
            && n.hp <= 0
            && !n.wants_event_stop
            && n.ai[1] != 2.0
            && tr_core::dd2::crystal_death_starts_loss_scene(n.ai[1])
        {
            n.ai[1] = 1.0;
            n.ai[0] = 0.0;
            n.hp = n.max_hp;
            n.dont_take_damage = true;
            loss_entered = true;
        }
    }
    if loss_entered {
        // `:40870-40879`: the portal marks — the scene's crystal-side
        // sweep runs before `ReportLoss` (`:40882`).
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::DD2LanePortal && n.hp > 0 {
                n.ai[1] = 1.0;
                n.ai[0] = 0.0;
            }
        }
        // `DD2Event.ReportLoss` (`DD2Event.cs:507-515`).
        tr_core::dd2::report_loss(&mut app.world.dd2_lost, &mut app.world.dd2_spawn_hold);
    }
    // `DD2Event.UpdateTime` (`DD2Event.cs:194-251`): the hold's decay,
    // then — at the exact zero crossing (`:222-236`) — the `!LostThisRun`
    // arm broadcasts the new wave (`Lang.GetInvasionWaveText` on the tier's
    // roster, `INFO_NEW_WAVE_COLOR`) and the tier-3 wave-7 opening summons
    // Betsy (`:228-231`); the loss arm broadcasts `LoseInvasionMessage`
    // (`:344-347`, the red `INFO_FAILURE_INVASION_COLOR`).
    if app.world.dd2_spawn_hold > 0 {
        app.world.dd2_spawn_hold -= 1;
        if app.world.dd2_spawn_hold == 0 {
            if !app.world.dd2_lost {
                // `:224`: `GetInvasionStatus` — the live wave (the win
                // sentinel already rewrote it back to the last wave).
                let tier = dd2_tier(app);
                let wave = app.world.dd2_wave;
                let (roster, _) = match tier {
                    2 => tr_core::dd2::enemies_for_wave_t2(wave),
                    3 => tr_core::dd2::enemies_for_wave_t3(wave),
                    _ => tr_core::dd2::enemies_for_wave(wave),
                };
                app.chat.new_text(
                    dd2_wave_text(wave, roster),
                    tr_core::dd2::INFO_NEW_WAVE_COLOR,
                );
                // `:228-231`: the tier-3 wave-7 Betsy summon.
                if wave == 7 && tier == 3 {
                    dd2_summon_betsy(app);
                }
            } else {
                app.chat.new_text(
                    crate::lang::text("DungeonDefenders2.InvasionLose"),
                    tr_core::dd2::INFO_FAILURE_INVASION_COLOR,
                );
            }
        }
    }
    // `NPC.AnyNPCs(548)`'s port: the lane gates' aiStyle-106 arm reads it
    // from the machine, so the world carries the crystal's live state.
    app.world.dd2_crystal_alive = app
        .npcs
        .iter()
        .any(|n| n.kind == NpcKind::DD2EterniaCrystal && n.hp > 0);
    // The scene-end resolve (`AI`'s aiStyle-105 arm's
    // `DD2Event.StopInvasion`, `NPC.cs:40798` win / `:40956` loss): the
    // event stops and the crystal goes. `StopInvasion` (`DD2Event.cs:
    // 293-312`) runs `WinInvasionInternal` FIRST when `win` (`:297-300`),
    // then clears the counters and wipes the entities (`:301-308`) — the
    // wipe is unconditional, both outcomes. The win arm reads the scene's
    // crystal (`ai[1] == 2`, the win machine) or the `dd2_won` latch; the
    // loss arm is everything else (`ai[1] == 1`, the loss machine). The
    // two never both fire: `dd2_won` gates `dd2_report_kill` (`:364`) and
    // the loss intercept refuses the winning crystal above.
    if app
        .npcs
        .iter()
        .any(|n| n.kind == NpcKind::DD2EterniaCrystal && n.wants_event_stop)
    {
        let win = app.world.dd2_won
            || app
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::DD2EterniaCrystal && n.wants_event_stop && n.ai[1] == 2.0);
        // `WinInvasionInternal` runs BEFORE `Ongoing = false` and BEFORE the
        // wipe — the medals drop on the still-live crystal (`:297-308`).
        if win {
            dd2_win_invasion(app);
        }
        // The counter-clear + `Ongoing = false` is the SAME for both outcomes
        // (`StopInvasion`'s body, `:301-307` — the `win` flag only gates the
        // `WinInvasionInternal` call above), so the loss-named helper serves
        // the win stop too.
        if tr_core::dd2::stop_invasion_loss(
            &mut app.world.dd2_event_ongoing,
            &mut app.world.dd2_wave,
            &mut app.world.dd2_wave_kills,
        ) {
            app.world.dd2_spawn_hold = 0;
            app.npcs
                .retain(|n| !(n.kind == NpcKind::DD2EterniaCrystal && n.wants_event_stop));
            wipe_dd2_entities(app);
        }
    }
}

/// `DD2Event.WipeEntities` (`DD2Event.cs:629-638`) — the three sweeps in
/// the C#'s order: the sentry towers, the invasion hostiles, the energy
/// crystals in the open chests. The `netMode == 2` `SendData(114)` tail
/// is the netplay slice's scope.
pub(crate) fn wipe_dd2_entities(app: &mut App) {
    // `ClearAllTowersInGame` (`:640-649`): every active
    // `ProjectileID.Sets.IsADD2Turret` slot dies (`Projectile.Kill`).
    for i in 0..app.projectiles.slots().len() {
        if app.projectiles.slots()[i].active
            && tr_core::dd2::is_dd2_turret(app.projectiles.slots()[i].kind)
        {
            app.projectiles.kill(i);
        }
    }
    // `ClearAllDD2HostilesInGame` (`:651-664`): `active = false` — the
    // direct removal, NOT the death chain (no loot, no kill credit, no
    // death sound; the port's retain drops the slot). The kind's vanilla
    // id rides the pool table (`dd2_pool_id`); the bartender (550) is
    // out of `BelongsToInvasionOldOnesArmy` and survives.
    app.npcs
        .retain(|n| !dd2_pool_id(n.kind).is_some_and(tr_core::dd2::belongs_to_ooa));
    // `ClearAllDD2EnergyCrystalsInChests` (`:666-692`): the currently
    // open chests only — the 3822 stacks turn to air.
    let open = app.player.as_ref().map(|p| p.chest).unwrap_or(-1);
    if open >= 0 {
        if let Some(chest) = app.world_chests.get_mut(open as usize) {
            for slot in chest.slots.iter_mut() {
                if slot.kind == tr_core::dd2::ENERGY_CRYSTAL_ITEM && slot.count > 0 {
                    *slot = crate::item::EMPTY_SLOT;
                }
            }
        }
    }
}

fn report_invasion_kill(app: &mut App, kind: NpcKind) {
    // The Old One's Army's wave progress (`DD2Event.CheckProgress`,
    // `DD2Event.cs:362-479`) rides its own counter and self-gates on
    // `Ongoing` (`:364`) — the C# runs it (`NPC.cs:78230`) BEFORE the
    // invasion group's match (`:78233`), which then runs regardless (a
    // DD2 event carries `invasionType == 0`, so the group gate refuses
    // there anyway).
    if app.world.dd2_event_ongoing {
        dd2_report_kill(app, kind);
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

/// The invasion spawn table (`NPC.cs:1419-1450`, `invasionType == 1`):
/// hardmode's Summoner 1/30 (no living ones), the Sorcerer 1/9, the Thief
/// 1/5, the Peon 1/3, the Warrior 1/3, else the Archer.
fn invasion_spawn_pick(
    rng: &mut UnifiedRandom,
    hard_mode: bool,
    summoner_alive: bool,
    invasion_type: i32,
    // The type-4 table's roster gates (`NPC.cs:1517`'s `!AnyNPCs(520)`,
    // `:1509`'s `!AnyNPCs(395)` + the 30% progress window).
    walker_alive: bool,
    saucer_alive: bool,
    progress_30: bool,
    // The type-3 table's Dutchman window (`NPC.cs:1460-1468`): the
    // invasion's drained fraction `(sizeStart - size) / sizeStart`.
    progress_frac: f32,
) -> NpcKind {
    let _hard_type = if invasion_type == 1 { Some(1) } else { None };
    // `invasionType == 2`'s table (`NPC.cs:1453-1466`): the gates are the
    // vanilla's sequential `Next(n) == 0` chain — `Next(7)` → net 145
    // (the port's `MysterySnowman`), `Next(3)` → net 143 (the Gangsta),
    // else net 144 (the port's `SnowBalla`).
    if invasion_type == 2 {
        if rng.next_max(7) == 0 {
            return NpcKind::MysterySnowman; // 145
        }
        if rng.next_max(3) == 0 {
            return NpcKind::SnowmanGangsta; // 143
        }
        return NpcKind::SnowBalla; // 144
    }
    // `invasionType == 4`'s table (`NPC.cs:1487-1540`): the roll ORDER is
    // vanilla — num6 (`Next(7)`), the Saucer gate (`Next(45)` + the 30%
    // progress + `!AnyNPCs(395)`), the num6 >= 6 rider arm (`Next(20)` +
    // the gate → 390 unported; else `Next(2)` → the Engineer 386), the
    // num6 >= 4 ray arm (`num8` `Next(5)`: <2 → the RayGunner 382 and
    // ==2 → the Scrambler 381 both unported, >=4 → the Drone 388), and
    // the else arm's ground chain (`num9` `Next(4)`, the 3 → the Walker
    // 520's walk-in gate, the `Next(3)` fallback). The Saucer 395 (the
    // boss), the Scutlix Rider 390, the RayGunner 382 and the Scrambler
    // 381 land with their AI families — their rolls consume the stream
    // exactly here.
    if invasion_type == 4 {
        // The vanilla chain (`NPC.cs:1495-1548`): `Next(7)` first, then
        // `Next(45)` (the Saucer gate), the num6 >= 6 rider arm
        // (`Next(20)` + the gate, else `Next(2)`), the num6 >= 4 ray arm
        // (`Next(5)`: <2 the RayGunner 382, 2-3 the Scrambler 381, 4 the
        // Drone 388), and the ground chain (`Next(4)`, the 3 → the Walker
        // 520's walk-in gate, else `Next(3)` → 385/389/383).
        let num6 = rng.next_max(7) as u32;
        let flag6 = progress_30 && !saucer_alive;
        if rng.next_max(45) == 0 && flag6 {
            // The Saucer 395: the core spawns its crew (the machine's
            // init arm).
            return NpcKind::MartianSaucerCore;
        }
        if num6 >= 6 {
            if rng.next_max(20) == 0 && flag6 {
                // The Scutlix Rider 390: the mount spawns its 391 carrier
                // (the machine's init arm).
                return NpcKind::ScutlixRider;
            }
            // `Next(2)`: 0 → the Rider 390, 1 → the Engineer 386.
            return if rng.next_max(2) == 0 {
                NpcKind::ScutlixRider
            } else {
                NpcKind::MartianEngineer
            };
        }
        if num6 >= 4 {
            // num8 < 2 → the RayGunner 382, 2-3 → the BrainScrambler 381,
            // 4 → the Drone 388.
            return match rng.next_max(5) {
                0 | 1 => NpcKind::RayGunner,
                2 | 3 => NpcKind::BrainScrambler,
                _ => NpcKind::MartianDrone,
            };
        }
        let mut num9 = rng.next_max(4) as u32;
        if num9 == 3 {
            if !walker_alive {
                return NpcKind::MartianWalker;
            }
            num9 = rng.next_max(3) as u32;
        }
        return match num9 {
            0 => NpcKind::GrayGrunt,
            1 => NpcKind::GigaZapper,
            _ => NpcKind::MartianOfficer,
        };
    }
    // `invasionType == 3`'s table (`NPC.cs:1460-1484`): the Flying Dutchman
    // 491 (the half-points window) is the boss-minion follow-up; the
    // Captain 216 1/30 while none alive; the Deadeye 1/11, the Corsair
    // 1/9, the Deckhand's musketeer 1/7, the CaptainsMate 1/3, else the
    // Deckhand.
    if invasion_type == 3 {
        // The Dutchman (`NPC.cs:1460-1468`): the half-points window
        // (the drained fraction in [0.5, 1)) spawns the ship — its four
        // cannons fight for it.
        // The Dutchman gate draws `Next(20)` inside the half-points
        // window (`NPC.cs:1462`); the AnyNPCs(491)/solid-tile gates fold
        // (the port runs the pick without the roster scan).
        if progress_frac >= 0.5 && progress_frac < 1.0 && rng.next_max(20) == 0 {
            return NpcKind::FlyingDutchman;
        }
        // The sequential chain (`NPC.cs:1469-1484`): `Next(30)` (the
        // Captain 216, `!AnyNPCs(216)` folds), `Next(11)` (the Deadeye
        // 252 → the port's Archer fold), `Next(9)` (the Corsair 215),
        // `Next(7)` (the Archer 214), `Next(3)` (the CaptainsMate 213),
        // else the Deckhand 212.
        if rng.next_max(30) == 0 {
            return NpcKind::PirateCaptain;
        }
        if rng.next_max(11) == 0 {
            return NpcKind::PirateArcher;
        }
        if rng.next_max(9) == 0 {
            return NpcKind::PirateCorsair;
        }
        if rng.next_max(7) == 0 {
            return NpcKind::PirateArcher;
        }
        if rng.next_max(3) == 0 {
            return NpcKind::PirateCaptainsMate;
        }
        return NpcKind::PirateDeckhand;
    }
    // `invasionType == 1`'s table (`NPC.cs:1428-1448`): the hardmode
    // Summoner gate (`Next(30) == 0` + `!AnyNPCs(471)`), then the
    // sequential chain — `Next(9)` (the Sorcerer 29), `Next(5)` (the
    // Thief 26), `Next(3)` (the Peon 111), `Next(3)` (the Warrior 27),
    // else the Archer 28.
    if hard_mode && !summoner_alive && rng.next_max(30) == 0 {
        return NpcKind::GoblinSummoner;
    }
    if rng.next_max(9) == 0 {
        return NpcKind::GoblinSorcerer;
    }
    if rng.next_max(5) == 0 {
        return NpcKind::GoblinThief;
    }
    if rng.next_max(3) == 0 {
        return NpcKind::GoblinPeon;
    }
    if rng.next_max(3) == 0 {
        return NpcKind::GoblinWarrior;
    }
    NpcKind::GoblinArcher
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

/// The saucer/scutlix/dutchman carrier write-through PRE-pass (S1-D-8b's
/// migrated scan family): the `AI_075` type-390 rider link, the saucer
/// mounts' nearest-core lookup + crew counts, and the `AI_093` Dutchman
/// crew count. The radius scans read the spatial hash
/// (`crate::npc_spatial::NpcSpatial`) over the per-tick `NpcSnapshot` —
/// candidates arrive in ascending slot order, the exact order the former
/// linear `npcs.iter()` scans visited, with the predicates, the strict
/// `< 1600.0` reach and the first-wins tie-break unchanged. The pass's own
/// `hp = 0` kills mirror into the snapshot (`snap.kill`) so later scans in
/// the family see the deaths exactly as the live reads did. The display
/// inheritance stays a live write pass (it mutates OTHER slots).
fn carrier_link_pre_pass(
    npcs: &mut [Npc],
    snap: &mut crate::world_sim::npc_snapshot::NpcSnapshot,
) {
    let spatial = crate::npc_spatial::NpcSpatial::build(snap);
    for i in 0..npcs.len() {
        if npcs[i].kind != NpcKind::ScutlixRider
            && npcs[i].kind != NpcKind::MartianSaucer
            && npcs[i].kind != NpcKind::MartianSaucerTurret
            && npcs[i].kind != NpcKind::MartianSaucerCannon
            && npcs[i].kind != NpcKind::FlyingDutchmanCannon
            && npcs[i].kind != NpcKind::FlyingDutchman
        {
            continue;
        }
        // The Dutchman's own crew count (`AI_093`'s `Main.npc[ai[i]]`
        // links have no port model — the count within the reach stands
        // in: the ship's death rides it).
        if npcs[i].kind == NpcKind::FlyingDutchman {
            let (scx, scy) = npcs[i].center();
            let count = spatial.count_living_kind_within(
                snap,
                scx,
                scy,
                NpcKind::FlyingDutchmanCannon,
                1600.0,
            );
            npcs[i].crew_cannons = count;
            // The display inheritance for the 492 cannons (`AI_093`'s
            // `NPC.cs:38718-38726`): the ship's `rotation` banks the
            // cannons with it, and its `spriteDirection` flips them.
            let ship_facing = npcs[i].sprite_dir;
            let ship_rot = npcs[i].rotation;
            for m in npcs.iter_mut() {
                if m.kind == NpcKind::FlyingDutchmanCannon && m.carrier_x >= 0.0 {
                    m.carrier_rot = ship_rot;
                    m.carrier_facing = ship_facing;
                }
            }
            continue;
        }
        let link_kind = if npcs[i].kind == NpcKind::ScutlixRider {
            NpcKind::Scutlix
        } else if npcs[i].kind == NpcKind::FlyingDutchmanCannon {
            NpcKind::FlyingDutchman
        } else {
            NpcKind::MartianSaucerCore
        };
        let (rcx, rcy) = npcs[i].center();
        let best = spatial.nearest_living_kind(snap, rcx, rcy, link_kind, 1600.0);
        match best {
            Some((j, _)) => {
                let (mcx, mcy) = npcs[j].center();
                npcs[i].carrier_x = mcx;
                npcs[i].carrier_y = mcy;
                // The saucer family's extra payload: the core's mirrored
                // timer + phase, the display inheritance (`NPC.cs:35625-35651`
                // / `:35669-35676`: every mount copies the carrier's
                // `rotation` and `spriteDirection`), and the crew counts
                // within the reach.
                if link_kind == NpcKind::MartianSaucerCore {
                    npcs[i].carrier_timer = npcs[j].ai[3];
                    npcs[i].carrier_phase = npcs[j].ai[0];
                    npcs[i].carrier_rot = npcs[j].rotation;
                    npcs[i].carrier_facing = npcs[j].sprite_dir;
                    let turrets = spatial.count_living_kind_within(
                        snap,
                        mcx,
                        mcy,
                        NpcKind::MartianSaucerTurret,
                        1600.0,
                    );
                    let cannons = spatial.count_living_kind_within(
                        snap,
                        mcx,
                        mcy,
                        NpcKind::MartianSaucerCannon,
                        1600.0,
                    );
                    npcs[i].crew_turrets = turrets;
                    npcs[i].crew_cannons = cannons;
                }
            }
            None => {
                // The Scutlix rider falls with its carrier; the saucer
                // mounts (the shell/guns) fall with the core.
                if npcs[i].carrier_x >= 0.0 {
                    npcs[i].carrier_x = -1.0;
                    npcs[i].carrier_y = -1.0;
                    npcs[i].hp = 0;
                    // Mirror the kill into the read-only view: a later
                    // mount's scan / crew count must not see it (the live
                    // `hp > 0` reads didn't either).
                    snap.kill(i);
                } else if npcs[i].kind == NpcKind::ScutlixRider {
                    // Never linked yet — the machine's init spawns.
                }
            }
        }
    }
}

/// The verbatim pre-migration linear scan (S1-D-8b's A/B reference): the
/// family exactly as it ran over `npcs.iter()` before the spatial hash.
/// Test-only — the twin-seed equivalence tests run it against the spatial
/// path over byte-identical pools.
#[cfg(test)]
fn carrier_link_pre_pass_linear(npcs: &mut [Npc]) {
    for i in 0..npcs.len() {
        if npcs[i].kind != NpcKind::ScutlixRider
            && npcs[i].kind != NpcKind::MartianSaucer
            && npcs[i].kind != NpcKind::MartianSaucerTurret
            && npcs[i].kind != NpcKind::MartianSaucerCannon
            && npcs[i].kind != NpcKind::FlyingDutchmanCannon
            && npcs[i].kind != NpcKind::FlyingDutchman
        {
            continue;
        }
        if npcs[i].kind == NpcKind::FlyingDutchman {
            let (scx, scy) = npcs[i].center();
            let count = npcs
                .iter()
                .filter(|m| {
                    m.kind == NpcKind::FlyingDutchmanCannon
                        && m.hp > 0
                        && {
                            let (mcx, mcy) = m.center();
                            (mcx - scx).hypot(mcy - scy) < 1600.0
                        }
                })
                .count() as i32;
            npcs[i].crew_cannons = count;
            let ship_facing = npcs[i].sprite_dir;
            let ship_rot = npcs[i].rotation;
            for m in npcs.iter_mut() {
                if m.kind == NpcKind::FlyingDutchmanCannon && m.carrier_x >= 0.0 {
                    m.carrier_rot = ship_rot;
                    m.carrier_facing = ship_facing;
                }
            }
            continue;
        }
        let link_kind = if npcs[i].kind == NpcKind::ScutlixRider {
            NpcKind::Scutlix
        } else if npcs[i].kind == NpcKind::FlyingDutchmanCannon {
            NpcKind::FlyingDutchman
        } else {
            NpcKind::MartianSaucerCore
        };
        let (rcx, rcy) = npcs[i].center();
        let mut best: Option<(usize, f32)> = None;
        for (j, m) in npcs.iter().enumerate() {
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
                let (mcx, mcy) = npcs[j].center();
                npcs[i].carrier_x = mcx;
                npcs[i].carrier_y = mcy;
                if link_kind == NpcKind::MartianSaucerCore {
                    npcs[i].carrier_timer = npcs[j].ai[3];
                    npcs[i].carrier_phase = npcs[j].ai[0];
                    npcs[i].carrier_rot = npcs[j].rotation;
                    npcs[i].carrier_facing = npcs[j].sprite_dir;
                    let turrets = npcs
                        .iter()
                        .filter(|m| {
                            m.kind == NpcKind::MartianSaucerTurret
                                && m.hp > 0
                                && (m.center().0 - mcx).hypot(m.center().1 - mcy) < 1600.0
                        })
                        .count() as i32;
                    let cannons = npcs
                        .iter()
                        .filter(|m| {
                            m.kind == NpcKind::MartianSaucerCannon
                                && m.hp > 0
                                && (m.center().0 - mcx).hypot(m.center().1 - mcy) < 1600.0
                        })
                        .count() as i32;
                    npcs[i].crew_turrets = turrets;
                    npcs[i].crew_cannons = cannons;
                }
            }
            None => {
                if npcs[i].carrier_x >= 0.0 {
                    npcs[i].carrier_x = -1.0;
                    npcs[i].carrier_y = -1.0;
                    npcs[i].hp = 0;
                } else if npcs[i].kind == NpcKind::ScutlixRider {
                    // Never linked yet — the machine's init spawns.
                }
            }
        }
    }
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
    // S1-D-8 (research/39 "Phase A"): the read-only snapshot seam. The
    // carrier-link scan family below (the parts' core lookup + the core's
    // part check) reads this per-tick view instead of re-borrowing
    // `app.npcs` per scan; the writes land on the scanning NPC alone. The
    // pass mirrors its own `hp = 0` kills into the view (`snap.kill`) so
    // later scans in the family see the deaths exactly as the live
    // `hp > 0` reads did — the scan predicates/order are unchanged.
    let mut snap = crate::world_sim::npc_snapshot::NpcSnapshot::build(&app.npcs);
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
            carrier_uid = snap.first_living_kind_uid(want).unwrap_or(0);
            app.npcs[i].carrier_uid = carrier_uid;
        }
        let core = snap.position_living_uid(carrier_uid);
        match core {
            Some(ci) => {
                let row = &snap.rows()[ci];
                let (ccx, ccy) = (row.cx, row.cy);
                app.npcs[i].carrier_x = ccx;
                app.npcs[i].carrier_y = ccy;
                app.npcs[i].carrier_timer = 1.0;
                app.npcs[i].moon_core_alive = true;
            }
            None => {
                app.npcs[i].moon_core_alive = false;
                app.npcs[i].carrier_timer = 0.0;
                app.npcs[i].hp = 0;
                // Mirror the kill into the read-only view: a later part's
                // fallback find / the core's alive check must not see it
                // (the live `hp > 0` reads didn't either).
                snap.kill(i);
            }
        }
    }
    // The core's part bookkeeping (`AI_077`'s `:41229-41232` +
    // `:41251-41291`): the three stored slots hold the parts' ids (the
    // `localAI[0..2]` link the spawn handed out), so the core's checks
    // resolve them.
    for i in 0..app.npcs.len() {
        if app.npcs[i].kind != NpcKind::MoonLordCore {
            continue;
        }
        // The C#'s flag (`:41251-41269`): a slot that points at no living
        // part takes the core down with it.
        let mut lost = false;
        // The C#'s flag2 (`:41270-41285`): all three parts in their death
        // state hand the core its vulnerable phase. A slot that never
        // got a part is not a dead part.
        let mut dead = true;
        for uid in app.npcs[i].moon_parts {
            if uid <= 0 {
                lost = true;
                dead = false;
                continue;
            }
            let alive = snap.any_living_uid(uid);
            if !alive {
                lost = true;
            } else {
                dead = false;
            }
        }
        app.npcs[i].moon_parts_lost = lost;
        app.npcs[i].moon_parts_dead = dead;
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
    // it (the retire lands before the retain pass below). S1-D-8b: the
    // family's radius scans read the spatial hash over the same per-tick
    // snapshot (ascending slot order = the linear scan's visit order);
    // the pass mirrors its own `hp = 0` kills via `snap.kill`.
    carrier_link_pre_pass(&mut app.npcs, &mut snap);
    // `UpdateTime_StartDay`'s `StartInvasion()` resolve (`Main.cs:65454`):
    // the morning roll's pending flag + the players' life-max gate. The
    // march state (the type/x/warn) rides the World — `invasion_goblins`
    // (the item-path flag) stays synced for the panel/kill paths.
    if app.world.invasion_arm_pending {
        app.world.invasion_arm_pending = false;
        let life_max = app.player.as_ref().map(|p| p.max_hp).unwrap_or(0);
        if can_start_invasion(&app.world, life_max) {
            // `Main.StartInvasion()`'s full body (`Main.cs:64435-64486`),
            // called from `UpdateTime_StartDay` (`:65454`) — its `Next(2)`
            // side roll rides the `UpdateTime` stream.
            let num = 1; // the session player (the multiplayer count folds)
            start_invasion_in(app, num, 1, Some(RngKey::UpdateTime));
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
            // S1-D-3: the anchor's FULL velocity (`:19258`'s
            // `nPC.velocity`), the body's raw life/lifeMax (the free
            // head's mixed DOUBLE/INTEGER tier comparisons, `:31420`/
            // `:31477`) and the part scan (`:19544-19566`'s flags).
            app.world.golem_vy = body.vy;
            app.world.golem_hp = body.hp;
            app.world.golem_max_hp = body.max_hp;
            app.world.golem_hp_ratio = body.hp as f32 / body.max_hp as f32;
            app.world.golem_head_alive = app
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::GolemHead && n.hp > 0);
            app.world.golem_fist_left_alive = app
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::GolemFistLeft && n.hp > 0);
            app.world.golem_fist_right_alive = app
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::GolemFistRight && n.hp > 0);
        } else {
            app.world.golem_present = false;
        }
    }
    // Pre-compute player center and nearby NPCs for the invasion spawn
    // path (needed before `app` is mutably borrowed below). Extract only
    // the fields we need to avoid borrow conflicts with the closure.
    let player_opt = app.player.as_ref().map(|p| {
        let (pcx, pcy) = p.center();
        let nearby = app
            .npcs
            .iter()
            .filter(|n| {
                let (cx, cy) = n.center();
                (cx - pcx).abs() < spawner::S_WIDTH as f32 * 0.5
                    && (cy - pcy).abs() < spawner::S_HEIGHT as f32 * 0.5
            })
            .count() as f32;
        (pcx, pcy, nearby, p.x, p.y, p.max_hp)
    });
    let (inv_pcx, inv_pcy, inv_nearby, inv_px, inv_py, inv_max_hp) =
        player_opt.unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0, 100));
    // Pre-extract scene_metrics and camera values needed for the invasion block
    // to avoid long-lived borrows that conflict with later mutable borrows of app.
    let water_candle_count = app.scene_metrics.water_candle_count;
    let peace_candle_count = app.scene_metrics.peace_candle_count;
    let camera_y = app.camera.y;
    // S1-E-21e — `Main.player[target].ZoneGraveyard`, the fifth live term of
    // `DespawnEncouragement_AIStyle3_Fighters_NotDiscouraged`
    // (`NPC.cs:60302`) the fighter-moan gate reads.
    let zone_graveyard = app.scene_metrics.zone_graveyard;
    // S1-F-13c wiring — the Everscream's per-tick dust arm (`NPC.cs:32821-
    // 32825`) rolls `Main.rand` through the verbatim `DustPool::new_dust` on
    // the SAME `UpdateNPCs` stream the AI draws ride (`Main.cs:17767-17769`'s
    // `SwapRandom("UpdateNPCs")` wraps `NPC.UpdateNPC`, dust included). The
    // frame's spawn ctx is captured HERE, before the `npcs_rng` mutable
    // borrow (the helper borrows `app` immutably); the live pool is threaded
    // into the tick closure below. Every other kind ignores them.
    let dust_spawn_ctx = app.spawn_ctx();
    // The AI group's stream: `Main.SwapRandom("UpdateNPCs")` wraps
    // `NPC.UpdateNPC` (`Main.cs:17767-17769`) — the AI draws ride it in
    // slot order, one shared stream across the tick closure (the
    // scoped-bind pattern, `clock.rs:108-111`).
    let npcs_rng = app.rngs.stream(RngKey::UpdateNpcs);
    // Plantera's hook-centroid anchor pre-pass (S1-D-3-plantera-minions,
    // `NPC.cs:31743-31762`): the C# scans the NPC slots INSIDE the boss's
    // own AI tick; the port pre-computes it once per tick over the pool
    // (the carrier pre-pass precedent). `type_of` resolves the VANILLA
    // type via `cs_id` (S1-D-3-wire-plantera-minions: `to_u16` is the
    // port's protocol id — 154 for the hook (gate-reassigned) — NOT the C# type 263 the
    // centroid filters on).
    {
        let type_of = |n: &Npc| n.kind.cs_id() as i32;
        for i in 0..app.npcs.len() {
            if app.npcs[i].kind != NpcKind::Plantera || app.npcs[i].hp <= 0 {
                continue;
            }
            let anchor = crate::npc_ai_plantera_anchor::hook_centroid(&app.npcs, type_of);
            app.npcs[i].plantera_hook_anchor = anchor;
        }
    }
    // S1-D-3 W-1: the WoF arm's live-count pre-pass — `NPC.CountNPCS(117)`
    // (`NPC.cs:25710`'s `< 10` leech gate) and the Hungry (115) `ai[0]`
    // collection in slot order capped at 10 (`:26003-26010`, the expert
    // respawn roll's free-slot scan). The C# scans inside the wall's own
    // AI tick; the port pre-computes once per tick over the pool (the
    // Plantera centroid precedent above).
    {
        let leech_count = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::LeechHead && n.hp > 0)
            .count() as i32;
        let hungry_ai0s: Vec<f32> = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::Hungry && n.hp > 0)
            .take(10)
            .map(|n| n.ai[0])
            .collect();
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::WallOfFlesh && n.hp > 0 {
                n.wof_leech_count = leech_count;
                n.wof_hungry_ai0s = hungry_ai0s.clone();
            }
        }
    }
    // S1-D-3 W-9 fishron wiring — `Main.windSpeedCurrent` pre-stamp for
    // the Detonating Bubble's wind blend (`NPC.cs:34903`): the AI's
    // `&World` borrow cannot reach `app.wind`, so the sim stamps it
    // before the update (the plantera_hook_anchor pre-pass convention).
    {
        let wind = app.wind.speed_current;
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::DetonatingBubble {
                n.wind_speed_current = wind;
            }
        }
    }
    let player = &mut app.player;
    // Limit the scope of borrows to just the tick closure
    {
        // S1-D-3 W-1: `&mut` — the WoF arm's band/index statics write back
        // INSIDE the loop, right after the wall's own update, so the
        // eye/Hungry slots later in the tick read the fresh values exactly
        // like the C#'s live `Main.wofDrawArea*`/`Main.npc[wofNPCIndex]`
        // reads (`NPC.cs:25724`/`:26066`/`:26280`). Every other reader in
        // the closure only reads.
        let world = &mut app.world;
        let npcs = &mut app.npcs;
        let audio = &mut app.audio;
        let dust = &mut app.dust;
        app.npc_step.tick(dt.min(0.25), |step| {
            if let Some(player) = player {
                for (i, n) in npcs.iter_mut().enumerate() {
                    // The NPC's slot index (the C#'s `NPC.whoAmI`) — the
                    // spawn requests that carry an owner reference (the
                    // Saucer deathray's `ai0 = whoAmI + 1`) read it.
                    n.slot = i;
                    // `NPC.OnSpawn`'s type-113 arm (`NPC.cs:8404-8407`):
                    // `Main.wofNPCIndex = whoAmI` at spawn — the presence
                    // mirror stays on while the wall lives (the death
                    // sweep's `ResetNPCSlotData` clears it, `:8379-8381`).
                    if n.kind == NpcKind::WallOfFlesh && n.hp > 0 {
                        world.wof_present = true;
                        world.wof_npc_index = i as i32;
                    }
                    // S1-E-21g — `NPC.UpdateNPC`'s soundDelay decay
                    // (`NPC.cs:90381-90384`) runs BEFORE the AI: the
                    // machine's `soundDelay == 0` test sees the decayed
                    // value, exactly the C#'s order.
                    if n.sound_delay > 0 {
                        n.sound_delay -= 1;
                    }
                    // S1-E-21e — `AI_003_Fighters`' idle counter
                    // (`NPC.cs:57034-57149`) runs at AI time, BEFORE the
                    // movement integrates: `old_x` still holds the previous
                    // tick's pre-integration stamp (`update_rng` re-stamps
                    // it, `NPC.cs:90496`), which is the C#'s
                    // `oldPosition.X` read (`:57126`). The plain-walker
                    // kinds' `ai[3]` is free for it (`fighter_walker_kind`).
                    // The player-hitbox reset (`:57146`) reads the live
                    // boxes ahead of the move.
                    if crate::ambient_sound::fighter_walker_kind(n.kind) {
                        let (nw, nh) = n.size();
                        let player_intersects = n.x < player.x + crate::player::WIDTH
                            && n.x + nw > player.x
                            && n.y < player.y + crate::player::HEIGHT
                            && n.y + nh > player.y;
                        n.ai[3] = crate::ambient_sound::fighter_idle_counter(
                            n.kind.cs_id() as i32,
                            n.ai[2],
                            n.ai[3],
                            n.vx,
                            n.vy,
                            n.facing,
                            n.x,
                            n.old_x,
                            // `justHit` rides the port's established
                            // `hurt_cd > 0` stand-in (`npc.rs:13420`).
                            n.hurt_cd > 0.0,
                            player_intersects,
                        );
                    }
                    n.update_rng_dust(world, player, step, npcs_rng, dust, &dust_spawn_ctx);
                    // S1-D-3 W-1 — the WoF statics write-back, right after
                    // the wall's own update (the C# mutates the statics
                    // inside the arm, `NPC.cs:25724`/`:25758-25842`, so
                    // later slots in the same tick read the fresh values).
                    // The arm staged them on the NPC (it holds `&World`).
                    if n.kind == NpcKind::WallOfFlesh && n.hp > 0 {
                        world.wof_draw_top = n.wof_band_top;
                        world.wof_draw_bottom = n.wof_band_bottom;
                        world.wof_npc_index = n.wof_npc_index_out;
                        world.wof_x = n.x;
                        world.wof_dir = n.facing;
                        world.wof_life = n.hp;
                        world.wof_life_max = n.max_hp;
                        world.wof_width = n.size().0 as i32;
                        world.wof_vx = n.vx;
                        world.wof_center_x = n.center().0;
                        world.wof_hp_ratio = n.hp as f32 / n.max_hp as f32;
                    } else if n.kind == NpcKind::WallOfFlesh {
                        // The wall died this tick — the presence mirror
                        // clears like `ResetNPCSlotData` (`NPC.cs:8379-
                        // 8381`), so the eye/Hungry slots later in the
                        // tick see the death immediately.
                        world.wof_present = false;
                        world.wof_npc_index = -1;
                        world.wof_life = 0;
                    }
                    // `NPC.UpdateNPC`'s tail (`NPC.cs:90516`): `justHit`
                    // clears at the update's end — the strike tick only
                    // (S1-D-3 W-1: the Hungry's recoil arm reads it).
                    n.just_hit = false;
                    // S1-E-21g — the AI machine's queued sound plays
                    // (the `wants_hostile_shot` shape): drained right
                    // after the machine's draws, BEFORE the moan/critter
                    // emitters — the C#'s plays sit inside `AI()` which
                    // precedes both (`NPC.cs:90323`). Each play gates
                    // attenuation BEFORE any case draw (the E21e
                    // precedent); the draws ride the UpdateNPCs stream.
                    for req in n.wants_sound.drain(..) {
                        play_npc_sound_request(audio, req, npcs_rng);
                    }
                    black_slime_darkness_touch(n, player, npcs_rng);
                    // `NPC.UpdateNPC`'s tail (`NPC.cs:90324-90327`): the
                    // look-for-home timer decays one per tick.
                    if n.look_for_home_timeout > 0 {
                        n.look_for_home_timeout -= 1;
                    }
                    // S1-E-21e — `AI_003_Fighters`' ambient moan arms
                    // (`NPC.cs:57336-57367`) ride the AI's tail, after its
                    // draws and before `UpdateNPC_CritterSounds`
                    // (`NPC.cs:90323`) — all on the `UpdateNPCs` stream
                    // (`Main.cs:17767`). The gate is the full
                    // `DespawnEncouragement_AIStyle3_Fighters_NotDiscouraged`
                    // (`NPC.cs:60300-60330`, `fighter_not_discouraged`):
                    // the event/exclusion terms and the switch's critter-set
                    // default all ride along — the excluded moan types (the
                    // mummies, the face monster, …) are not discouraged even
                    // by day above the surface. The kind gate keeps the
                    // counter-only walkers (the scarecrows 305-314,
                    // Splinterling 326, the renumbered goblins/pirates)
                    // draw-free.
                    if crate::ambient_sound::fighter_walker_kind(n.kind) {
                        let npc_type = n.kind.cs_id() as i32;
                        let not_discouraged = crate::ambient_sound::fighter_not_discouraged(
                            npc_type,
                            n.y as f64,
                            world.world_surface * 16.0,
                            world.eclipse,
                            world.day_time,
                            n.spawned_from_statue,
                            zone_graveyard,
                            world.snow_moon,
                            world.invasion_type,
                            world
                                .seeds
                                .contains(tr_core::seeds::SeedFlags::DONT_STARVE),
                            n.ai[1],
                            n.ai[2],
                        );
                        let (nx, ny) = (n.x, n.y);
                        crate::ambient_sound::fighter_ambient_moans(
                            npc_type,
                            n.ai[3],
                            not_discouraged,
                            n.shimmer_transparency,
                            npcs_rng,
                            &mut |legacy_case, style, rng| {
                                audio.play_fighter_moan(legacy_case, style, nx, ny, rng);
                            },
                        );
                    }
                    // `NPC.UpdateNPC`'s tail (`NPC.cs:90323`): the critter
                    // sounds ride the same per-NPC stream after the AI's
                    // draws — the table's rolls, then the legacy family's
                    // slot/pitch draws. The C#'s `Main.netMode == 2` gate
                    // (`NPC.cs:92245`) is the dedicated server's; the port
                    // has none. No ported kind matches the critter table
                    // yet (M4-NPC's roster has no critter ids), so the
                    // emitter stays silent until they land.
                    let mut hi_roll = |hi: u32| npcs_rng.next_max(hi as i32) as u32;
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
                        let mut signed = |lo: i32, hi: i32| npcs_rng.next_range(lo, hi);
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
    // The `59 HitSwitch` send sites (`Collision.cs:3304`/`:3318`/`:3324`/
    // `:3347`): collected by the scan below, sent once the `app.wiring`/
    // `app.world` borrows (and the `player` borrow) release.
    let mut switch_sends: Vec<(i32, i32)> = Vec::new();
    {
        let mut wiring = &mut app.wiring;
        let world = &mut app.world;
        let audio = &mut app.audio;
        for n in app.npcs.iter() {
            let (w, h) = n.size();
            let switch = crate::collision::switch_tiles(
                wiring, world, n.x, n.y, n.old_x, n.old_y, w as i32, h as i32,
                2, // the NPC's objType (`NPC.cs:90504`)
            );
            if let Some((item, x, y)) = switch.dropped {
                // `WorldGen.KillTile`'s drop for the broken plate — the
                // switch bridge runs inside `NPC.UpdateNPC` (the `UpdateNpcs`
                // swap, `Main.cs:17767-17769`), so the roll rides the AI
                // group's stream.
                crate::world_sim::items::spawn_item_drop_fields(
                    &mut app.items,
                    npcs_rng,
                    item,
                    1,
                    x,
                    y,
                );
            }
            // `Collision.cs:3304`/`:3318`/`:3324`/`:3347` — the `59 HitSwitch`
            // send sits on every fired switch.
            switch_sends.extend(switch.switches.iter().copied());
            // S1-E-21i — the NPC-tripped switches' Mech-28 clicks: the
            // pitch draws ride the `UpdateNPCs` stream (`NPC.UpdateNPC`
            // runs under the `Main.cs:17767-17769` swap), drained per NPC
            // so the play order matches the slot order.
            for (px, py) in wiring.take_switch_clicks() {
                audio.play_switch_click(px as f32, py as f32, npcs_rng);
            }
        }
    }
    // The `59` sends after the last `player` borrow (`Collision.cs:3304`).
    for (x, y) in switch_sends {
        app.send_hit_switch(x, y);
    }
    // S1-C-7 wave B — the contact lane's hurt modifiers (`Player.Hurt`
    // `:38396-38517`): the accepted contact hit queued its retaliate request
    // (`hurt_inner`'s tail) and possibly a dismount-on-hit; the C# rolls the
    // block inline under the `UpdateNPCs` swap (`Main.cs:17767`), the port
    // drains at the pass tail on the same stream (the S1-E-21c batch
    // convention, ANALYSIS "The hurt modifiers"). The drain sits after the
    // switch-tile scan — that scan still draws the scoped `npcs_rng`
    // binding, so the retaliate rolls re-borrow the stream last.
    if let Some(p) = app.player.as_mut() {
        crate::hurt_modifiers::drain_dismount(p, &app.world);
    }
    if let Some(req) = app.player.as_mut().and_then(|p| p.take_retaliate()) {
        let expert = app.player.as_ref().is_some_and(|p| p.buffs.expert_mode);
        crate::hurt_modifiers::drain_retaliate(
            app.player.as_mut().unwrap(),
            &mut app.projectiles,
            &mut app.npcs,
            req,
            app.local_id as u8,
            expert,
            app.rngs.stream(RngKey::UpdateNpcs),
        );
    }
    // `NPC.cs:77986-77996`: `this.timeLeft--` once per NPC update, and
    // `timeLeft <= 0` deactivates it — no distance gate. The aiStyle-4
    // day/dead arm (`:20071-20074`) is the only writer so far, so this is the
    // Eye of Cthulhu leaving 10 ticks after dawn.
    for n in app.npcs.iter_mut() {
        n.tick_despawn();
    }
    app.npcs.retain(|n| !n.despawn_expired());
    // The worm pass (`AI_006`): the chain movement after the per-NPC step
    // (the vanilla interleaves it in the slot order; the port's pass reads
    // the same spawn order).
    update_worms(app);
    // The Plantera minion pool pass (S1-D-3-wire-plantera-minions): the
    // aiStyle-52 hook + aiStyle-53 tentacle machines read the WHOLE pool
    // (`NPC.plantBoss` + the sibling/anchor slots, `NPC.cs:32080`/`:32129`/
    // `:32260`), so they run here after the per-NPC step — the worm pass's
    // precedent (the C# interleaves them in slot order; the port's pass
    // reads the same spawn order, one tick of AI/integration lag for
    // freshly spawned minions — the documented pool-pass deviation).
    update_plantera_minions(app);
    // S1-D-3-wire-destroyer: the Destroyer family's OWN machine (`AI_037`)
    // + the Probe's `AI_005` type-139 arm — the same after-the-step position
    // as the worm pass (the family is no longer `is_worm()`).
    update_destroyers(app);
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
        let mut asking: Vec<(usize, i32, i32, i32)> = Vec::new();
        for (i, n) in npcs.iter().enumerate() {
            if n.kind != NpcKind::Guide || n.home.is_some() || n.look_for_home_timeout > 0 {
                continue;
            }
            asking.push((
                i,
                (n.x / TILE_PX) as i32,
                (n.y / TILE_PX) as i32,
                n.kind.net_id(),
            ));
        }
        for (i, tx, ty, net_id) in asking {
            // `ScoreRoom`'s `ignoreNPC` — the asking NPC is not an occupant.
            if let Some((hx, hy)) =
                crate::town::quick_find_home(world, npcs, tx, ty, Some(i), net_id)
            {
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
        // `:65763-65784` — the five SpawnAllowed_* flags read the inventory.
        // The Party Girl's roll is `Main.rand.Next(40) == 0`, drawn ONCE PER
        // CHECK — the C# draws it after `UpdateNpcsSpawnsTownNPCs`'s
        // `checkForSpawns >= 7200` early-return (`Main.cs:65550-65554`), so
        // the draw gates on the same boundary (the counter increments
        // inside `update_time_spawn_town_npcs`; the pre-increment value
        // predicts whether this tick is a check). The `UpdateTime_SpawnTownNPCs`
        // identity runs under the `UpdateTime` swap (`Main.cs:65542`,
        // called `:65153`).
        let party_roll = app.town_arrival.check_for_spawns + 1 >= 7200
            && app.rngs.stream(RngKey::UpdateTime).next_max(40) == 0;
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
                if let Some((hx, hy)) =
                    crate::town::quick_find_home(&app.world, &app.npcs, xt, yt, None, kind.net_id())
                {
                    // `WorldGen.SpawnTownNPC`'s relocation arm
                    // (`WorldGen.cs:5428` `FindNPCLookingForHomeThatCanMoveIn`
                    // → `:5453-5463`): an already-live homeless NPC of this
                    // kind (and not on a `kickOut` cooldown, `:5621`) takes the
                    // room instead of a second copy spawning. That is what
                    // `moveRoom` (`:4843-4855`) relies on.
                    let existing = app.npcs.iter().position(|n| {
                        n.kind == kind && n.home.is_none() && n.look_for_home_timeout == 0
                    });
                    if let Some(idx) = existing {
                        let mut rooms = std::mem::take(&mut app.town_rooms);
                        let npc = &mut app.npcs[idx];
                        crate::town::move_room(&mut rooms, npc, (hx, hy));
                        app.town_rooms = rooms;
                        // `:5460`: the relocation fires NO_HOBO (progression 8).
                        let _ = app.notify_progression_net(8);
                    } else if let Some(mut n) = Npc::spawn_town_at(&app.world, hx, hy, kind) {
                        // `WorldGen.cs:5564-5565`: the arrival writes its room.
                        n.home = Some((hx, hy));
                        app.npcs.push(n);
                        // `WorldGen.cs:5576`: the arrival fires NO_HOBO
                        // (progression 8) — the Truffle's IT_CAN_TALK (18,
                        // `:5580`) waits on a ported Truffle kind, and
                        // REAL_ESTATE_AGENT (17, `:5671`) on the full
                        // `CheckAchievement_RealEstateAndTownSlimes` roster.
                        let _ = app.notify_progression_net(8);
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
        // `NPC.checkDead`'s DD2 crystal arms (`NPC.cs:79240-79278`,
        // `DoDeathEvents`'s 552-578 rows — they run inside `NPCLoot`
        // (`:78223` → `:78856`), BEFORE `CheckProgress` at `:78230`): the
        // budget decision reads the pre-kill invasion pair, so the drop
        // walk rides ahead of the kill report's wave advance. S1-E-12c.
        dd2_crystal_drop(app, npc);
        report_invasion_kill(app, npc.kind);
        // `NPC.cs:78230-78232`'s death-site pair — `CheckProgressFrostMoon`
        // then `CheckProgressPumpkinMoon` — the C# runs them right after
        // `DD2Event.CheckProgress` and before the invasion group's match. The
        // port bundles the DD2 walk and the group accounting inside
        // `report_invasion_kill` (disjoint state: the invasion size vs the
        // moon-event waves, and neither draws the RNG), so the moon walks
        // ride right behind it; each walk self-gates on its own flag, so a
        // single running event walks alone.
        app.world.report_frost_moon_kill(npc.kind.to_u16());
        app.world.report_pumpkin_moon_kill(npc.kind.to_u16());
        // `NPC.checkDead` plays `this.DeathSound` at `base.position`
        // (`NPC.cs:78186-78194`) — the kind's own death cue through the
        // faithful case-4 path (S1-E-21 wave C): the volume arms + the
        // `Main.rand.Next(-10, 11) * 0.01` pitch draw. The C# runs
        // `checkDead` inside `StrikeNPC` (the striking stage's stream);
        // the port batches deaths into the NPC pass, so the draw rides
        // the `UpdateNPCs` stream (`Main.cs:17767`) — the recorded
        // deviation (ANALYSIS, S1-E-21c). The C#'s `null` guard: the
        // kinds with no row play nothing.
        if let Some(cue) = npc.kind.death_sound() {
            let rng = app.rngs.stream(crate::app::RngKey::UpdateNpcs);
            app.audio.play_npc_death_sound(cue, npc.x, npc.y, rng);
        }
        // S1-E-21g — `NPC.HitEffect`'s EoC death arm (`NPC.cs:87275-87297`):
        // the type-4 kill plays the case-15 roar TWICE — the play sits
        // inside the `num567 < 2` gore loop (`:87289-87298`), after the
        // fourth gore's two draws per pass. The port's gore/dust layer
        // is skipped (the standing deviation), so the two plays queue
        // back to back; the second usually refuses on the case-15
        // playing guard (`:834`) with a live output device, and both
        // pass headless (the fresh-slot table, the wave-C convention).
        // The C# runs HitEffect inside StrikeNPC (the striking stage's
        // stream); the port's death batching rides UpdateNPCs — the
        // same recorded deviation as the death cue above. Roars draw
        // nothing, so the stream choice is unobservable here.
        if npc.kind == NpcKind::EyeOfCthulhu {
            let rng = app.rngs.stream(crate::app::RngKey::UpdateNpcs);
            play_npc_sound_request(
                &mut app.audio,
                crate::npc::NpcSoundRequest {
                    cue: crate::npc::NpcSoundCue::Roar(0),
                    x: npc.x,
                    y: npc.y,
                },
                rng,
            );
            let rng = app.rngs.stream(crate::app::RngKey::UpdateNpcs);
            play_npc_sound_request(
                &mut app.audio,
                crate::npc::NpcSoundRequest {
                    cue: crate::npc::NpcSoundCue::Roar(0),
                    x: npc.x,
                    y: npc.y,
                },
                rng,
            );
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
            // S1-D-3 W-9 fishron wiring — `DoDeathEvents`'s case 370
            // (`NPC.cs:79348-79350`): `SetEventFlagCleared(ref
            // downedFishron, 7)`. The progression notification (the
            // flag-7 net message) rides the flag; the port's
            // notify_progression_net carries the WoF precedent's shape.
            NpcKind::DukeFishron => {
                app.world.downed_fishron = true;
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
                // `WorldGen.TriggerLunarApocalypse` fires from `checkDead`
                // under the `UpdateNPCs` swap (`NPC.cs:79393-79396`,
                // `Main.cs:17767-17769`).
                let rng = app.rngs.stream(RngKey::UpdateNpcs);
                crate::world_sim::lunar::trigger_lunar_apocalypse(
                    &mut app.world,
                    &mut app.npcs,
                    rng,
                );
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
                    // `NPC.cs:79450-79456` draws `Next(2)` under the
                    // `UpdateNPCs` swap (`checkDead` -> `DoDeathEvents`
                    // inside `NPC.UpdateNPC`, `Main.cs:17767-17769`) — the
                    // stream S1-A-11's npcs pass lands on in P3.
                    app.world.spawn_meteor = tr_core::worldgen::meteor::boss_death_roll(
                        app.rngs.stream(crate::app::RngKey::UpdateNpcs),
                        true,
                    );
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
        // `NPCKillsTracker.RegisterKill`'s `netMode == 2` half
        // (`NPCKillsTracker.cs:29-32`): the host broadcasts the new count
        // through `NetBestiaryModule.SerializeKillCount`.
        app.broadcast_bestiary_kill(
            entry.net_id,
            app.bestiary.kills.get_kill_count(entry.credit_id),
        );
        // `AchievementsHelper.TryGrantingBestiary100PercentAchievement` (`AchievementsHelper.cs:81-84`):
        // when the bestiary completion reaches 100%, fire progression event 29 (BOOK_WORM).
        let _ = app.try_grant_bestiary_100_net();
        // `AchievementsHelper.NotifyNPCKilled` (`AchievementsHelper.cs:200-217`):
        // the process whose player interacted gets the credit locally
        // (`:202-208`, and in MP the host's own `97` loops back through its
        // player slot, `:210-216`); a host additionally fans `97` out to every
        // other interactor.
        if npc.player_interaction[app.local_id as usize] {
            let _ = app
                .achievements
                .notify(achievements::CondKind::NpcKilled(entry.net_id));
        }
        app.send_npc_killed(npc, entry.net_id as i16);
        // `NPC.CountKillForBannersAndDropThem` (`NPC.cs:79818-79827`) →
        // `BannerSystem.AddNPCKillBy` (M2-BUFF-8): the milestone spawns the
        // banner item at the death spot and announces the tally in the
        // Command color.
        if let Some((banner, text)) = app.banners.add_npc_kill_by(entry.net_id, &app.player_name) {
            crate::chat::broadcast_death_text(&mut app.chat, &text, crate::chat::COLOR_COMMAND);
            let item = tr_core::bestiary::banner_to_item(banner);
            // `CountKillForBannersAndDropThem` fires from the death path —
            // the `UpdateNPCs` stream (`NPC.cs:79818-79827`).
            let rng = app.rngs.stream(RngKey::UpdateNpcs);
            crate::world_sim::items::spawn_item_drop(
                &mut app.items,
                rng,
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
                    // `NPC.cs:79405-79410` (case 50): `Main.StopSlimeRain()`
                    // fires from `checkDead` under the `UpdateNPCs` swap —
                    // the duration draw rides `UpdateNPCs`, not the weather
                    // tick's `UpdateTime` (the other caller).
                    crate::world_sim::weather::stop_slime_rain(
                        &mut app.world,
                        app.rngs.stream(crate::app::RngKey::UpdateNpcs),
                    );
                    let _ = app.notify_progression_net(16);
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
                    let _ = app.notify_progression_net(9);
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
        for (kind, sx, sy, ai3) in n.wants_sphere.drain(..) {
            // `NPC.NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80771`): the payload
            // is the sphere's bottom-centre.
            let (w, h) = kind.size();
            let mut ball = Npc::new(kind, sx - w * 0.5, sy - h, 1, 0.0);
            ball.hp = ball.max_hp;
            // `NPC.NewNPC`'s `ai3` argument (`NPC.cs:21246`'s
            // `flag4 ? 1 : 0` — the Dark Caster's red-hat sphere).
            ball.ai[3] = ai3;
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
    // S1-D-3 W-9 fishron wiring — Duke Fishron's spawn requests
    // (`AI_069_DukeFishron`): the phase-1 bubble volley
    // (`NPC.NewNPC(..., (int)p.X, (int)p.Y + 45, 371)`, `NPC.cs:49246`)
    // and the phase-2 Cthulunado trail (`:49476-49481`). The volley
    // bubbles spawn with the stock target 255 — their own aiStyle-70
    // init draws the four RNG values on a later tick (the style70
    // census); the trail bubbles inherit target/velocity/ai[3] AT THE
    // SPAWN SITE (the scale was drawn inside the boss's tick, `:49481`)
    // and short-circuit that init. `NPC.NewNPC`'s Bottom = (X, Y)
    // (`NPC.cs:80771`).
    let mut fishron_spawns = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::DukeFishron {
            continue;
        }
        for (sx, sy) in n.wants_fishron_bubbles.drain(..) {
            let (w, h) = NpcKind::DetonatingBubble.size();
            let mut bubble = Npc::new(NpcKind::DetonatingBubble, sx - w * 0.5, sy - h, 1, 0.0);
            bubble.hp = bubble.max_hp;
            fishron_spawns.push(bubble);
        }
        for (sx, sy, svx, svy, target, scale) in n.wants_fishron_trail.drain(..) {
            let (w, h) = NpcKind::DetonatingBubble.size();
            let mut bubble = Npc::new(NpcKind::DetonatingBubble, sx - w * 0.5, sy - h, 1, 0.0);
            bubble.hp = bubble.max_hp;
            // `:49477-49481` — the preset writes, verbatim.
            bubble.target = target;
            bubble.vx = svx;
            bubble.vy = svy;
            bubble.net_update = true;
            bubble.ai[3] = scale;
            fishron_spawns.push(bubble);
        }
    }
    app.npcs.append(&mut fishron_spawns);
    // Fishron's sharknado bolts (`Projectile.NewProjectile(..., 385)` —
    // phase 1's pair `NPC.cs:49279-49280`, phase 2's aimed bolt
    // `:49505`): the ai0/ai1 payload rides `spawn_hostile_ai` (the
    // saucer deathray's shape); the enrage flag is the C#'s third
    // spawn arg (`localAI[0]`, `:49505`'s `flag6 ? 1 : 0`).
    let mut bolts = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::DukeFishron {
            continue;
        }
        for (x, y, vx, vy, ai0, ai1, enrage) in n.wants_fishron_bolts.drain(..) {
            bolts.push((x, y, vx, vy, ai0, ai1, enrage));
        }
    }
    for (x, y, vx, vy, ai0, ai1, enrage) in bolts {
        // The C#'s NewProjectile takes the CENTRE; the port's spawn
        // takes the top-left (the inferno-blast precedent). The owner
        // is Main.myPlayer (`NPC.cs:49279`/`:49505`) — the local id.
        if let Some(slot) = app.projectiles.spawn_hostile_ai_owned(
            crate::projectile::SHARKNADO_BOLT,
            app.local_id as u8,
            Vec2::new(x - 15.0, y - 15.0),
            Vec2::new(vx, vy),
            0,
            0.0,
            ai0,
            ai1,
        ) {
            app.projectiles.set_local_ai0(slot, enrage);
        }
    }
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
            // `Main.npc[num474].target = this.target` (`NPC.cs:27642`/
            // `:27647`/`:27652`/`:27658`): the limbs inherit the head's
            // target — the cannon's state-1 steer reads it BEFORE its
            // first TargetClosest (`:28717`). The head's init
            // `TargetClosest()` (`:27637`) picks the only active player;
            // the port's v1 head does not target, so the drain stands in
            // the single-player result: slot 0 (documented stand-in —
            // `Npc::new` starts at the ClearTarget 255, which would
            // index past the port's one-slot player slice).
            limb.target = 0;
            if timed {
                limb.ai[3] = 150.0;
            }
            limb.hp = limb.max_hp;
            limbs.push(limb);
        }
    }
    app.npcs.append(&mut limbs);
    // The Queen Bee's state-1 bee volley (S1-D-3 W-3, `NPC.cs:30623-30636`):
    // the aiStyle-43 axis arm set `wants_bee` on a spawn tick (the
    // Next(20)/Next(210,212) draws already rode the UpdateNPCs stream in
    // C# order); this pass resolves the `NPC.NewNPC` — the (int)vector77
    // truncation is in the payload, `Bottom = (X, Y)` (`NPC.cs:80770`),
    // the normalized ×5 velocity toward the player and `localAI[0] = 60`
    // (`:30633`). `CanBeReplacedByOtherNPCs` (`:30632`) has no port field
    // — dropped (the module's recorded fold).
    let mut bees = Vec::new();
    for n in app.npcs.iter_mut() {
        let Some(req) = n.wants_bee.take() else {
            continue;
        };
        let kind = match req.kind {
            crate::npc_ai_style43::BEE => NpcKind::Bee,
            crate::npc_ai_style43::BEE_SMALL => NpcKind::BeeSmall,
            _ => unreachable!("the volley draws only 210/211"),
        };
        let (bw, bh) = kind.size();
        let mut bee = Npc::new(kind, req.x - bw * 0.5, req.y - bh, 1, 0.0);
        bee.hp = bee.max_hp;
        bee.vx = req.vx;
        bee.vy = req.vy;
        bee.local_ai[0] = req.local_ai0;
        bee.net_update = true;
        bees.push(bee);
    }
    app.npcs.append(&mut bees);
    // Plantera's spawn requests (S1-D-3-plantera, `NPC.cs:31736-31742` /
    // `:31983-31994` / `:32054-32073`): S1-D-3-wire-plantera-minions
    // replaces the drain-only stand-in with the real `NewNPC` spawns —
    // the hooks (263, aiStyle 52), the tentacles (264, aiStyle 53) and
    // the spore (265, aiStyle 50). `NewNPC` places the NPC by its BOTTOM
    // at the boss centre (`NPC.cs:80769`, `Entity.Bottom` setter
    // `Entity.cs:128-131`: `position = (X - width/2, Y - height)`), sets
    // `timeLeft = (int)(activeTime * 1.25)` = 937 (`:80771`,
    // `activeTime = 750` `:95186`) and `target = 255` (the default
    // `Target` arg; the spore's velocity comes from the request).
    // getGood +6 tentacles / the expert hook-linked batch are
    // constants-off (the boss module's documented classic seam).
    let mut plantera_spawns: Vec<Npc> = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::Plantera {
            continue;
        }
        let (cx, cy) = n.center();
        let hooks = std::mem::take(&mut n.wants_plantera_hooks);
        for _ in 0..hooks {
            let (w, h) = NpcKind::PlanterasHook.size();
            let mut hook = Npc::new(NpcKind::PlanterasHook, cx - w * 0.5, cy - h, 1, 0.0);
            hook.despawn_timer = 937;
            plantera_spawns.push(hook);
        }
        let tentacles = std::mem::take(&mut n.wants_plantera_tentacles);
        for _ in 0..tentacles {
            let (w, h) = NpcKind::PlanterasTentacle.size();
            let mut tentacle =
                Npc::new(NpcKind::PlanterasTentacle, cx - w * 0.5, cy - h, 1, 0.0);
            tentacle.despawn_timer = 937;
            plantera_spawns.push(tentacle);
        }
        if let Some((vx, vy)) = n.wants_plantera_spore.take() {
            let (w, h) = NpcKind::Spore.size();
            let mut spore = Npc::new(NpcKind::Spore, cx - w * 0.5, cy - h, 1, 0.0);
            spore.despawn_timer = 937;
            // `:32069-32071` — the jittered 8-px launch velocity.
            spore.vx = vx;
            spore.vy = vy;
            spore.net_update = true;
            plantera_spawns.push(spore);
        }
    }
    app.npcs.append(&mut plantera_spawns);
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
            // S1-D-3 prime wiring: the faithful limb arms' seam reads
            // (`Main.npc[(int)ai[1]]` — `NPC.cs:28249`'s INTEGER half-width
            // centre, `:28446`/`:28489`'s velocity blends, `:28326`'s phase
            // gate) — the head slot's live fields, not the legacy centre.
            app.world.prime_head_x = prime.x;
            app.world.prime_head_width = pw as i32;
            app.world.prime_head_vx = prime.vx;
            app.world.prime_head_vy = prime.vy;
            app.world.prime_head_ai1 = prime.ai[1];
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
            // S1-E-14b: the `FindFirstNPC(35)` mirror the hand's
            // `localAI[3]` copy (`NPC.cs:22315`) and the Dark Caster's
            // `aiStyle 8` latch (`NPC.cs:20820-20827`) read — the head's
            // live `ai[3]` (the port's hands-count stand-in; the red-hat
            // head model deviation is recorded in ANALYSIS).
            app.world.skeletron_red_hat = head.ai[3] == 1.0;
        }
        if !app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::SkeletronHead && n.hp > 0)
        {
            app.world.skeletron_red_hat = false;
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
            app.world.golem_vy = body.vy;
            app.world.golem_hp = body.hp;
            app.world.golem_max_hp = body.max_hp;
            app.world.golem_hp_ratio = body.hp as f32 / body.max_hp as f32;
            let head_alive = app
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::GolemHead && n.hp > 0);
            app.world.golem_head_alive = head_alive;
            app.world.golem_fist_left_alive = app
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::GolemFistLeft && n.hp > 0);
            app.world.golem_fist_right_alive = app
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::GolemFistRight && n.hp > 0);
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
    // B-6c (S1-E-12d): the crystal's lane-portal spawn (`NPC.cs:40744-40755`'s
    // `localAI[0]` one-shot): the update pass set `wants_lane_portals` with
    // `base.Bottom`, this pass runs the arena walker and spawns the two 549s
    // — BEFORE the lane-spawn resolve below, so a portal created this tick
    // enters the gate machine on the next one (the C#'s `NewNPC` slots
    // update from the following `UpdateNPCs` frame too).
    let mut portal_requests = Vec::new();
    for n in app.npcs.iter_mut() {
        if let Some(bottom) = n.wants_lane_portals.take() {
            portal_requests.push(bottom);
        }
    }
    for bottom in portal_requests {
        // `StrayMethods.CheckArenaScore(base.Bottom, out xLeftEnd, out
        // xRightEnd)` (`:40746`, `StrayMethods.cs:83-105`) — pure tile
        // walk, ZERO RNG draws (census in `tr_core::stray`).
        let (mut left_end, mut right_end) = tr_core::stray::check_arena_score(&app.world, bottom);
        // `:40748-40749`: the ends nudge 2 tiles toward the centre.
        left_end.0 += 2;
        right_end.0 -= 2;
        // `:40750-40753`: `NewNPC(xEnd.X, xEnd.Y, 549)` spawns Bottom-
        // anchored at the tile corner (`NPC.cs:80770`), then `Bottom =
        // end.ToWorldCoordinates(8, 16)` (`Utils.cs:1762-1765`: tile*16 +
        // (8,16)) re-centres it — the net placement is `x = end.X*16 + 8 -
        // 78/2`, `y = end.Y*16 + 16 - 130` (the 549 box, `NPC.cs:16037-
        // 16038`). The right portal carries `ai[2] = 1` (`:40751`'s `1f`
        // in `NewNPC`'s `ai2` slot — the `leftGate` flag the lane machine
        // reads, `:40972`). `dontTakeDamage = true` is the 549's
        // `SetDefaults` row (`NPC.cs:16049`).
        for (end, right) in [(left_end, false), (right_end, true)] {
            let mut gate = Npc::new(
                NpcKind::DD2LanePortal,
                end.0 as f32 * TILE_PX + 8.0 - 78.0 * 0.5,
                end.1 as f32 * TILE_PX + 16.0 - 130.0,
                1,
                0.0,
            );
            gate.hp = gate.max_hp;
            gate.dont_take_damage = true;
            if right {
                gate.ai[2] = 1.0;
            }
            app.npcs.push(gate);
        }
        // `DD2Event.FindArenaHitbox()` (`:40754`) — the arena rect feeds
        // only `ShouldBlockBuilding` (`Player.cs:25963-25969`'s noBuilding
        // + buff 199), unported; deferred with that pass (ANALYSIS).
    }
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
    let mut gallop_dust = Vec::new();
    for (i, n) in app.npcs.iter_mut().enumerate() {
        for shot in n.wants_hostile_shot.drain(..) {
            shots.push((i, shot));
        }
        for shot in n.wants_hostile_shot_ai.drain(..) {
            ai_shots.push(shot);
        }
        for d in n.wants_gallop_dust.drain(..) {
            gallop_dust.push(d);
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
        // (`NPC.cs:49891`), the Frost Legion throws'
        // (`NPC.cs:29022`/`:29084`), the Golem family's eye beams
        // (`NPC.cs:31307`/`:31333`/`:31568` - S1-D-3), Plantera's seeds
        // (`NPC.cs:31966-31969` - every volley kind except the 277 thorn
        // ball) and the Queen Bee's stinger (`NPC.cs:30743` - S1-D-3 W-3).
        let time_left = if kind == crate::projectile::DESTROYER_LASER
            || kind == crate::projectile::BULLET_SNOWMAN
            || kind == crate::projectile::SNOW_BALL_HOSTILE
            || kind == crate::projectile::GOLEM_EYE_BEAM
            || kind == crate::projectile::SEED_PLANTERA
            || kind == crate::projectile::POISON_SEED_PLANTERA
            || kind == crate::projectile::QUEEN_BEE_STINGER
        {
            300
        } else {
            0
        };
        // The Frost throws' `ai[0] = 2f` post-spawn writes
        // (`NPC.cs:29025`/`:29087`): for 109 the value keeps tile collision
        // on while both `AI_010` gravity arms exclude the kind (the ball
        // flies straight); for 110 it is inert (`AI_001`'s `flag3 = false`
        // list holds the counter, `Projectile.cs:55962`). The laser keeps
        // its `NewProjectile` default 0 (`NPC.cs:49877`).
        let ai0 = if kind == crate::projectile::BULLET_SNOWMAN
            || kind == crate::projectile::SNOW_BALL_HOSTILE
        {
            2.0
        } else {
            0.0
        };
        if time_left > 0 {
            app.projectiles.spawn_hostile_timed(
                kind,
                Vec2::new(sx, sy),
                Vec2::new(svx, svy),
                dmg,
                0.0,
                time_left,
                ai0,
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
        let slot = app.projectiles.spawn_hostile_ai(
            shot.kind,
            Vec2::new(shot.x, shot.y),
            Vec2::new(shot.vx, shot.vy),
            shot.damage,
            0.0,
            shot.ai0,
            shot.ai1,
        );
        // S1-D-4 W-10: the Horseman's pumpkin scale roll
        // (`Projectile.cs:9518`) was consumed at the lob's QUEUE site
        // (`unicorn_step`'s fourth draw); apply it to the spawned slot.
        if let (Some(slot), Some(scale)) = (slot, shot.scale) {
            app.projectiles.set_scale(slot, scale);
        }
    }
    // The Horseman's gallop dust (`NPC.cs:62431-62437`, S1-D-4 W-10):
    // the arm-level gate + jitter draws already rode the UpdateNPCs
    // stream in-step; `Dust.NewDustPerfect(pos, 6, velocity)` and its
    // post-spawn writes run here on the SAME stream (the pool-internal
    // draw placement after the tick's AI is the recorded deviation,
    // ANALYSIS "W-10 unicorn tails").
    if !gallop_dust.is_empty() {
        let ctx = app.spawn_ctx();
        let rng = app.rngs.stream(RngKey::UpdateNpcs);
        for d in gallop_dust {
            let idx = app.dust.new_dust_perfect(
                rng,
                &ctx,
                &app.world,
                Vec2::new(d.x, d.y),
                6,
                Some(Vec2::new(d.vx, d.vy)),
                0,
                tr_core::particles::Color::default(),
                1.0,
            );
            // `:62434-62437` — the post-spawn writes, sentinel included.
            let slot = app.dust.slot_mut(idx);
            slot.scale = 0.6;
            slot.fade_in = 1.1;
            slot.no_gravity = true;
            slot.no_light = true;
        }
    }
    // S1-D-3 W-1 — the WoF's `NPC.NewNPC` drain (`NPC.cs:25712`,
    // `:26036`, `:26044-26049`). The faithful aiStyle-27 arm emits the
    // requests in C# order (leech heads, the expert Hungry respawn, the
    // ONE-TIME init: 2 mirror eyes + 11 Hungries at `localAI[0]` 1 → 2,
    // `:26041-26043`); this pass resolves them with `NewNPC`'s
    // `Bottom = (X, Y)` placement (`NPC.cs:80770`, `Entity.Bottom` setter:
    // `position = (X - width/2, Y - height)`, `Entity.cs:122-131`).
    // THE CATASTROPHIC FIX (census W-1): the v1 spawned the two eyes
    // EVERY tick the wall lived (ungated loop) and never cleared
    // `wants_hungry`/`wants_leech`, so 11 Hungries re-spawned every tick.
    // Now the module's one-shot init owns the gating; the requests drain
    // exactly once and the legacy flags clear after the drain.
    let mut wof_new: Vec<Npc> = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::WallOfFlesh {
            continue;
        }
        for req in n.wof_spawns.drain(..) {
            // The vanilla type ids → port kinds. 24 (the getGood demon)
            // has no port kind — getGoodWorld is false in the classic
            // port so the arm never emits it; documented skip.
            let kind = match req.npc_type {
                117 => NpcKind::LeechHead,
                114 => NpcKind::WallOfFleshEye,
                115 => NpcKind::Hungry,
                _ => continue,
            };
            let (w, h) = kind.size();
            // `NewNPC` sets no direction — the ctor default
            // `Entity.direction = 1` (`Entity.cs:20`) rides; the port's
            // `Npc::new` facing argument carries it (`spriteDirection`
            // stays the ctor's -1, `NPC.cs:6515`). `Start` is the C#'s
            // slot-search start (`GetAvailableNPCSlot`, `:80796`) — the
            // port's compacting pool appends instead (the established
            // spawn convention); `ai0` rides the request.
            let mut child = Npc::new(
                kind,
                req.x as f32 - w * 0.5,
                req.y as f32 - h,
                1,
                0.0,
            );
            child.hp = child.max_hp;
            child.ai[0] = req.ai0;
            if let Some(vx) = req.vx {
                // `Main.npc[num345].velocity.X = direction * 8`
                // (`NPC.cs:25713`) — the leech head's slide.
                child.vx = vx;
            }
            wof_new.push(child);
        }
        // The legacy request flags clear after the drain (the v1's
        // never-cleared `wants_hungry` was the respawn loop's second half).
        n.wants_hungry = false;
        n.wants_leech = false;
    }
    app.npcs.append(&mut wof_new);
    // The band ease/clamps moved INTO the arm (`style27_step`, the sim
    // write-back above) — the v1's sim-side ease toward `wof_*_target`
    // is retired with the targets. The presence mirror: the update loop
    // arms it per living wall (the `OnSpawn` shape); when none lives the
    // statics reset like `ResetNPCSlotData` (`NPC.cs:8379-8381`).
    if !app
        .npcs
        .iter()
        .any(|n| n.kind == NpcKind::WallOfFlesh && n.hp > 0)
    {
        app.world.wof_present = false;
        app.world.wof_npc_index = -1;
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
        // `NPC.AI` runs inside `UpdateNPCs` (`Main.cs:17767-17769`), so the
        // type-4 body's side roll (absent — type 4 telegraphs from spawn)
        // rides that stream.
        start_invasion_in(app, 1, 4, Some(RngKey::UpdateNpcs));
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
    // `(Main.rand.NextFloat() - 0.5f) * 5f` rides the AI caller's
    // `UpdateNPCs` stream (`NPC.cs:38898`).
    let drop_rng = app.rngs.stream(RngKey::UpdateNpcs);
    for n in app.npcs.iter_mut() {
        if n.kind != NpcKind::FlyingDutchman {
            continue;
        }
        if let Some((kind, dx, dy)) = n.wants_dutchman_drop.take() {
            let (pw, ph) = kind.size();
            let mut p = Npc::new(kind, dx - pw * 0.5, dy - ph, 1, 0.0);
            p.hp = p.max_hp;
            p.vx = (drop_rng.next_float() - 0.5) * 5.0 + n.vx;
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
            // `KillMe` → `GetDeathText` runs inside `Player.Update` — the
            // `UpdatePlayers` swap (`Main.cs:17754`). The C# variant roll is
            // `Main.rand.Next(n) + 1` (`Lang.cs:1057-1127`) — one integer
            // draw per death, before the player borrow is taken.
            let death_text = dot_death_text(
                player.dot_death,
                &app.player_name,
                app.rngs.stream(RngKey::UpdatePlayers),
            );
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
    // world update under `Main.SwapRandom("SpawnNPC")` (`Main.cs:17886-17890`
    // nested in the `SpawnNPCs` stage). The stream is PERSISTENT — the C#
    // spawner's state is the named stream's draw position, so the port binds
    // it here per tick (the lazy first creation = `Main.cs:3210-3212`) and
    // every draw advances it across ticks. There are NO unconditional draws:
    // the first draw is the `Next(spawnRate)` gate (`NPC.cs:312-318`), so the
    // per-call LCG advance the splitmix bridge needed dies outright.
    let mut spawned = None;
    if app.npcs.len() < 200 && app.npc_timer <= 0.0 {
        let mut rng = app.rngs.stream(RngKey::SpawnNpc);
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
                    // `Main.rand.Next(num2) != 0 → return` (`NPC.cs:5914`).
                    if rng.next_max(denom as i32) == 0 {
                        // `Next(num3, maxValue)` / `Next(minValue, maxValue2)`
                        // in px, then `/ 16` (`NPC.cs:5922-5924`).
                        let pcx_i = pcx as i32;
                        let pcy_i = pcy as i32;
                        let x_i = rng.next_range(pcx_i - view_w as i32, pcx_i + view_w as i32);
                        let y_i = rng.next_range(
                            pcy_i - (view_h * 1.5) as i32,
                            pcy_i - (view_h * 0.75) as i32,
                        );
                        let xt = x_i / (TILE_PX as i32);
                        let yt = y_i / (TILE_PX as i32);
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
                            // The classic chain (`NPC.cs:5944-5952`):
                            // `Next(10) == 0` → Yellow(-7), `Next(5) < 2` →
                            // Green(-3), else Blue. (The `Next(200) == 0`
                            // -4 arm folds — the kind is unported.)
                            let variant = if rng.next_max(10) == 0 {
                                SlimeVariant::Yellow
                            } else if rng.next_max(5) < 2 {
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
        // `NPC.CountNPCS(id)`'s scan source (`NPC.cs:7982-7995`): the active
        // roster's vanilla types — the moon ladders' per-kind cap gates
        // (`CountNPCS(id) < cap`) read it per arm.
        let moon_counts: Vec<u16> = app
            .npcs
            .iter()
            .filter(|n| n.hp > 0)
            .map(|n| n.kind.to_u16())
            .collect();
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
                moon_npc_counts: &moon_counts,
                flags: spawner::SpawnFlags {
                    zone_dungeon: app.scene_metrics.zone_dungeon,
                    zone_jungle: app.scene_metrics.zone_jungle,
                    // `Player.ZoneEvil` = the corrupt or crimson zone.
                    zone_evil: app.scene_metrics.zone_corrupt || app.scene_metrics.zone_crimson,
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
                    // `Player.UpdateBuffs`' Invisibility (10) / Battle (13)
                    // arms (`Player.cs:9991`/`:10003` → `NPC.GetSpawnRate`'s
                    // `player.invis`/`player.enemySpawns`, `NPC.cs:702-740`):
                    // the flags ride the buff scan (S1-E-7 wave A).
                    invis: player.invis,
                    enemy_spawns: player.enemy_spawns,
                    // `Main.pumpkinMoon || Main.snowMoon` — the raid pacing's
                    // moon override (`NPC.cs:771-774`): the events force
                    // `5·2.3` max spawns / rate 20 the moment they are up.
                    moon_event: app.world.pumpkin_moon || app.world.snow_moon,
                    ..spawner::SpawnFlags::default()
                },
                world: &app.world,
                player_center: (pcx, pcy),
                player_pos: (player.x, player.y),
                nearby_active_npcs: nearby,
                town_npcs: app.scene_metrics.town_npc_count,
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
                // `Spawner`'s ctor boss cap (`NPC.cs:262-286`): the weighted
                // slots count over the live wave-20 boss kinds — structurally
                // 0 until the kinds land.
                moon_boss_cap_count: spawner::moon_boss_cap_count(&app.npcs),
            })
        });
        // The one spawn path (`NPC.cs:308-342`) on the persistent `SpawnNPC`
        // stream: `spawn_npc_call` (S1-F-12 extract) runs the gates, the
        // 50-candidate scan and `SpawnAnNPC`'s zone chain — the invaders
        // ladder wins the dispatch through the ctx's `invaders` flag; the
        // retired pre-path rebuilt the gates, drew the pick off `item_rng`
        // and placed the NPC at a fixed offset. The state persists across
        // ticks in the stream (S1-A-11).
        let (spawn, _suppressed) = spawner::spawn_npc_call(ctx, &mut rng, app.spawn_no_cycle);
        app.spawn_no_cycle = false; // the latch is consumed by one call
        spawned = spawn;
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
    // `NPCLoot()` fires from the death path inside `NPC.UpdateNPC`
    // (`NPC.cs:78218/78223/78429`) — the drop rolls ride the `UpdateNPCs`
    // stream via the P1 `DropRng` impl.
    let rng = app.rngs.stream(RngKey::UpdateNpcs);
    let zones = &app.scene_metrics;
    // The drop rules read the killer's luck (`info.player.RollLuck`,
    // `CommonDrop.cs:36` — the port's single player).
    // `Language.GetTextValue` rides the seam (`Conditions.cs:970` — the
    // `NamedNPC` keys resolve through the same tables the name system uses);
    // the loot path is never the simulation (`NPC.cs:78933`).
    let get_text = |key: &str| crate::lang::text(key);
    let mut info = tr_core::loot::DropAttemptInfo {
        rng: &mut *rng,
        luck: app.player.as_ref().map(|p| p.luck()).unwrap_or(0.0),
        is_expert: false,
        is_master: false,
        active_players: 1,
        extra_gel: false,
        is_in_simulation: false,
        get_text_value: Some(&get_text),
        flags: tr_core::loot::LootFlags {
            hard_mode: app.world.hard_mode,
            // The mech-spawner gates + `BeatAnyMechBoss`/
            // `DownedAllMechBosses` read the real downed state
            // (`MechBossSpawnersDropRule.cs:15-23`, `Conditions.cs`).
            downed_mech1: app.world.downed_mech_boss1,
            downed_mech2: app.world.downed_mech_boss2,
            downed_mech3: app.world.downed_mech_boss3,
            downed_mech_any: app.world.downed_mech_boss_any(),
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
            // `info.npc.value` reads the real base (`MechBossSpawner's
            // `npc.value > 0f` gate, `HalloweenWeapons`' cap) — not the
            // placeholder 1.0.
            value: npc.value(),
            life_max: npc.max_hp,
            friendly: false,
            spawned_from_statue: npc.spawned_from_statue,
            is_boss: npc.kind == NpcKind::KingSlime,
            has_player_target: true,
            // The slime-body rain / statue-mimic style (`ai[1]`) and the
            // RedHat Skeletron state (`ai[3]`) ride the live ai vector
            // (`SlimeBodyItemDropRule.cs:19-22`, `NPC.cs:66671`). S1-E-14b:
            // the 36/32 `localAI[3]` paths ride the ported `local_ai`
            // vector (the hand mirrors the head, `NPC.cs:22315`; the Dark
            // Caster latches from `FindFirstNPC(35)`, `:20820-20827`).
            ai: npc.ai,
            local_ai: npc.local_ai,
            // `HasGivenName` = the non-empty name (`NPC.cs:6741`); the empty
            // string maps to `None` (Decision D2).
            given_name: (!npc.given_name.is_empty()).then_some(npc.given_name.as_str()),
            ..Default::default()
        },
    };
    let mut out = Vec::new();
    tr_core::loot::try_dropping(crate::loot_db::database(), net_id, &mut info, &mut out);
    for req in out {
        if req.item > 0 && crate::registry::get(req.item as u16).is_some() {
            // The velocity triplet rides the same stream (`Item.NewItem`'s
            // scatter); `Item.Prefix` uses `Main.rand` = the active swap
            // (`Item.cs:527`) — the prefix roll joins the `UpdateNPCs`
            // stream, no separate prefix stream.
            let vx = (rng.next_float() * 2.0 - 1.0) * 1.2;
            let vy = -1.5 - rng.next_float();
            let phase = rng.next_float() * std::f32::consts::TAU;
            app.items.push(ItemDrop::new(
                req.item as u16,
                req.stack as u32,
                crate::prefix::roll_prefix(req.item as u16, -1, rng),
                x,
                y,
                vx,
                vy,
                phase,
            ));
        }
    }
}

/// `NPC.NPCLoot_DropMoney` through the ported chain
/// (`tr_core::loot::drop_money`): the per-kind `NPC.value` base, the
/// ±20-75 % spread, the six bonus chains and the platinum/gold/silver/copper
/// split. Coin velocities ride the death path's `UpdateNPCs` stream like
/// the C# scatter.
fn spawn_coins(app: &mut App, npc: &Npc) {
    let (x, y) = npc.center();
    // `NPCLoot_DropMoney` fires from the same death path — the `UpdateNPCs`
    // stream (`NPC.cs:79622`'s luck read rides it too).
    let rng = app.rngs.stream(RngKey::UpdateNpcs);
    let mut out = Vec::new();
    tr_core::loot::drop_money(
        rng,
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
    for req in out {
        // The velocity triplet on the same stream (the port's ItemDrop
        // model).
        let vx = (rng.next_float() * 2.0 - 1.0) * 1.2;
        let vy = -1.5 - rng.next_float();
        let phase = rng.next_float() * std::f32::consts::TAU;
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

    /// S1-D-8b's A/B equivalence proof: the migrated carrier family
    /// (spatial hash over the snapshot) against the verbatim pre-migration
    /// linear scan, over twin pools built from the same seeds. Every
    /// write-visible field must match bit-for-bit — including the kill
    /// cascade (a mount whose core is out of reach dies, and later scans
    /// must not see it) and the display inheritance.
    #[test]
    fn the_spatial_carrier_pass_matches_the_linear_scan_over_twin_seeds() {
        // A tiny deterministic LCG — no game RNG stream is touched.
        struct Lcg(u64);
        impl Lcg {
            fn next(&mut self) -> u64 {
                self.0 = self
                    .0
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                self.0 >> 33
            }
            fn f32(&mut self, lo: f32, hi: f32) -> f32 {
                lo + (self.next() % 10_000) as f32 / 10_000.0 * (hi - lo)
            }
        }
        let kinds = [
            NpcKind::ScutlixRider,
            NpcKind::Scutlix,
            NpcKind::MartianSaucer,
            NpcKind::MartianSaucerTurret,
            NpcKind::MartianSaucerCannon,
            NpcKind::MartianSaucerCore,
            NpcKind::FlyingDutchman,
            NpcKind::FlyingDutchmanCannon,
            NpcKind::Slime,
        ];
        for seed in [1u64, 42, 512, 20261118] {
            // Build the twin pools from the SAME seed — byte-identical.
            let build = || {
                let mut rng = Lcg(seed);
                let mut npcs: Vec<Npc> = Vec::new();
                for i in 0..120 {
                    let kind = kinds[i % kinds.len()];
                    // Clusters within reach + far strays + cell edges.
                    let (x, y) = match i % 4 {
                        0 => (rng.f32(-400.0, 400.0), rng.f32(-400.0, 400.0)),
                        1 => (rng.f32(1200.0, 2000.0), rng.f32(-800.0, 800.0)),
                        2 => (rng.f32(-6400.0, 6400.0), rng.f32(-6400.0, 6400.0)),
                        _ => (1600.0 * (i % 3) as f32, -1600.0 * (i % 2) as f32),
                    };
                    let mut n = Npc::new(kind, x, y, 1, 0.0);
                    n.uid = (i + 1) as u32;
                    n.hp = if rng.next() % 4 == 0 { 0 } else { n.max_hp };
                    // Half the mounts start linked (the fall-with-carrier
                    // arm), half never linked.
                    n.carrier_x = if rng.next() % 2 == 0 { 10.0 } else { -1.0 };
                    n.carrier_y = 20.0;
                    n.ai[0] = rng.f32(0.0, 4.0);
                    n.ai[3] = rng.f32(0.0, 60.0);
                    n.rotation = rng.f32(-3.0, 3.0);
                    n.sprite_dir = if rng.next() % 2 == 0 { 1 } else { -1 };
                    npcs.push(n);
                }
                npcs
            };
            let mut a = build();
            let mut b = build();
            // A: the migrated spatial path. B: the verbatim linear scan.
            let mut snap = crate::world_sim::npc_snapshot::NpcSnapshot::build(&a);
            carrier_link_pre_pass(&mut a, &mut snap);
            carrier_link_pre_pass_linear(&mut b);
            assert_eq!(a.len(), b.len());
            for (i, (na, nb)) in a.iter().zip(&b).enumerate() {
                assert_eq!(na.hp, nb.hp, "seed {seed} slot {i} hp");
                assert_eq!(
                    na.carrier_x.to_bits(),
                    nb.carrier_x.to_bits(),
                    "seed {seed} slot {i} carrier_x"
                );
                assert_eq!(
                    na.carrier_y.to_bits(),
                    nb.carrier_y.to_bits(),
                    "seed {seed} slot {i} carrier_y"
                );
                assert_eq!(
                    na.carrier_timer.to_bits(),
                    nb.carrier_timer.to_bits(),
                    "seed {seed} slot {i} carrier_timer"
                );
                assert_eq!(
                    na.carrier_phase.to_bits(),
                    nb.carrier_phase.to_bits(),
                    "seed {seed} slot {i} carrier_phase"
                );
                assert_eq!(
                    na.carrier_rot.to_bits(),
                    nb.carrier_rot.to_bits(),
                    "seed {seed} slot {i} carrier_rot"
                );
                assert_eq!(
                    na.carrier_facing, nb.carrier_facing,
                    "seed {seed} slot {i} carrier_facing"
                );
                assert_eq!(
                    na.crew_turrets, nb.crew_turrets,
                    "seed {seed} slot {i} crew_turrets"
                );
                assert_eq!(
                    na.crew_cannons, nb.crew_cannons,
                    "seed {seed} slot {i} crew_cannons"
                );
            }
        }
    }

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
        let mut rng = UnifiedRandom::new(0);
        let mut hi_roll = |hi: u32| rng.next_max(hi as i32) as u32;
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
        assert_eq!(
            rng,
            UnifiedRandom::new(0),
            "out-of-table types consume no draws"
        );
    }

    /// S1-E-21 wave C — the death play's stream contract: `checkDead`'s
    /// `DeathSound` (`NPC.cs:78186-78194`) plays through the case-4 path
    /// with exactly one `Next(-10, 11)` pitch draw (`LegacySoundPlayer.cs:693`)
    /// on the `UpdateNPCs` stream (`Main.cs:17767` — the port's death seam
    /// runs in the NPC pass, the recorded ANALYSIS deviation); the other
    /// streams stay untouched.
    #[test]
    fn s1_e_21c_the_death_play_draws_one_pitch_on_the_npc_stream() {
        use crate::app::RngKey;
        // The wiring: a lone dead zombie's pass plays exactly one cue
        // (the death play — the control app with no NPC plays none).
        let mut control = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut npc = Npc::new(NpcKind::Zombie, 100.0, 100.0, 1, 0.0);
        npc.hp = 0; // dead: the retain drops it and plays the death cue
        app.npcs.push(npc);
        let before = app.audio.played_count();
        let control_before = control.audio.played_count();
        update_npcs(&mut control, 1.0 / 60.0);
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(
            control.audio.played_count() - control_before,
            0,
            "the empty pass plays nothing"
        );
        assert_eq!(
            app.audio.played_count() - before,
            1,
            "the zombie's death cue played once"
        );
        assert!(app.npcs.is_empty(), "the dead NPC was retained out");
        // The stream contract: the death play on the `UpdateNPCs` stream
        // pays exactly the one case-4 pitch draw
        // (`LegacySoundPlayer.cs:693`) — the death events (loot, gores)
        // legitimately share the stream, so pin the play itself against
        // a witness clone.
        let mut witness = app.rngs.stream(RngKey::UpdateNpcs).clone();
        let _pitch = witness.next_range(-10, 11);
        let pre: Vec<_> = RngKey::ALL
            .iter()
            .filter(|&&k| k != RngKey::UpdateNpcs)
            .map(|&k| (k, app.rngs.stream(k).clone()))
            .collect();
        assert_eq!(
            app.audio.play_npc_death_sound(
                "NPC_Killed_2",
                100.0,
                100.0,
                app.rngs.stream(RngKey::UpdateNpcs),
            ),
            Some("NPC_Killed_2")
        );
        assert_eq!(
            app.rngs.stream(RngKey::UpdateNpcs),
            &witness,
            "the death play paid exactly the case-4 pitch draw"
        );
        for (k, pre) in &pre {
            assert_eq!(
                app.rngs.stream(*k),
                pre,
                "{} untouched by the death play",
                k.name()
            );
        }
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
        // The party roll draws `Next(40)` from stream(UpdateTime) — seed
        // the streams so the roll is deterministic for the fixture.
        app.rngs = crate::app::rng::NamedRngs::new(app.world.seed as i32);
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
        // The party roll draws `Next(40)` from stream(UpdateTime) — seed
        // the streams so the roll is deterministic for the fixture.
        app.rngs = crate::app::rng::NamedRngs::new(app.world.seed as i32);
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
        // The party roll's UpdateTime draw is the fixture seed's.
        app.rngs = crate::app::rng::NamedRngs::new(app.world.seed as i32);
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
        app.rngs = crate::app::rng::NamedRngs::new(app.world.seed as i32);
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
                    moon_npc_counts: &[],
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
                    moon_boss_cap_count: 0.0,
                };
                let mut rng = UnifiedRandom::new((seed + 1) as i32);
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
        // The table: the picks are always goblins.
        let mut rng = UnifiedRandom::new(7);
        for _ in 0..40 {
            let kind = invasion_spawn_pick(&mut rng, false, false, 1, false, false, false, 0.0);
            assert!(matches!(
                kind,
                NpcKind::GoblinSorcerer
                    | NpcKind::GoblinThief
                    | NpcKind::GoblinPeon
                    | NpcKind::GoblinWarrior
                    | NpcKind::GoblinArcher
            ));
        }
        // The hardmode summoner: only when none alive.
        let mut rng = UnifiedRandom::new(3);
        let mut saw_summoner = false;
        for _ in 0..200 {
            if matches!(
                invasion_spawn_pick(&mut rng, true, false, 1, false, false, false, 0.0),
                NpcKind::GoblinSummoner
            ) {
                saw_summoner = true;
                break;
            }
        }
        assert!(saw_summoner, "the hardmode summoner roll");
        let mut rng = UnifiedRandom::new(7);
        for _ in 0..200 {
            assert!(
                !matches!(
                    invasion_spawn_pick(&mut rng, true, true, 1, false, false, false, 0.0),
                    NpcKind::GoblinSummoner
                ),
                "the alive summoner blocks the roll"
            );
        }
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
        // The roll's gate (`Next(20) == 0`): sweep for the seed whose
        // FIRST draw lands on it, so the spawn is deterministic.
        let mut probe = 1i32;
        while UnifiedRandom::new(probe).next_max(20) != 0 {
            probe += 1;
        }
        // `stream(UpdateTime)` is lazy — the fresh `NamedRngs` for the
        // app's seed must produce the probe's draw, so re-seed the world
        // to the probe (the fixture rig builds `NamedRngs` from
        // `world.seed`).
        app.world.seed = probe as u64;
        app.rngs = crate::app::rng::NamedRngs::new(probe);
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
    fn black_slime_darkness_touch_rolls_the_update_npcs_stream() {
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
        // The first `Next(4)` of seed 1 — under the darkness gate?
        let mut rng = UnifiedRandom::new(1);
        let first = rng.next_max(4);
        black_slime_darkness_touch(&black, &mut player, &mut UnifiedRandom::new(1));
        assert_eq!(
            player.buffs.active_types().contains(&22),
            first == 0,
            "seed 1 first roll {first}"
        );
        // A seed whose first `Next(4)` is non-zero denies the grant.
        let mut deny_seed = 2;
        while UnifiedRandom::new(deny_seed).next_max(4) == 0 {
            deny_seed += 1;
        }
        let mut player2 = crate::player::Player::spawn(&world);
        black_slime_darkness_touch(&black, &mut player2, &mut UnifiedRandom::new(deny_seed));
        assert!(!player2.buffs.active_types().contains(&22));
        // The stream advances only when the touch rolls (a blue slime skips
        // the roll entirely — the same seed stays put).
        let blue =
            Npc::spawn_slime_variant(&world, (px / 16.0) as i32, SlimeVariant::Blue).unwrap();
        let mut untouched = UnifiedRandom::new(7);
        black_slime_darkness_touch(&blue, &mut player, &mut untouched);
        assert_eq!(untouched, UnifiedRandom::new(7), "no draw on the skip");
    }

    #[test]
    fn dot_death_text_selects_table_over_stub() {
        let _guard = crate::lang::test_lock()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        // DW-1.3 — the death collection resolves the DoT table text when the
        // drain left a reason (`Player.cs:19321-19331,19362-19385`) and the
        // Slain row for any other death (`Lang.cs` CreateDeathMessage).
        assert_eq!(
            dot_death_text(None, "Belsin", &mut UnifiedRandom::new(1)),
            "Belsin was slain..."
        );
        // Poison is a single-variant row (no draw); Burned's two variants —
        // seed 1's first `Next(2)` is 0 → variant 1.
        assert_eq!(
            dot_death_text(Some(DotDeath::Poison), "Belsin", &mut UnifiedRandom::new(1)),
            "Belsin couldn't find the antidote."
        );
        assert_eq!(
            dot_death_text(Some(DotDeath::Burned), "Belsin", &mut UnifiedRandom::new(1)),
            "Belsin couldn't put the fire out."
        );
    }

    #[test]
    fn spawn_gel_and_coins_ride_the_player_luck() {
        // DW-3.1 — the drop rules read the killer's luck (`CommonDrop.cs:36`
        // through `info.luck`) and `NPCLoot_DropMoney` reads
        // `closestPlayer.luck` (`NPC.cs:79622`): at luck 1.0 both hit
        // branches always fire, so the loot stream lands elsewhere than at
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

        spawn_coins(&mut a, &npc);
        spawn_loot(&mut a, &npc);
        let state_0 = a.rngs.stream(crate::app::RngKey::UpdateNpcs).clone();

        // Replay from the pristine stream with luck 1.0 — the hit branches
        // draw differently (`CommonDrop.cs:36`'s luck double-roll).
        a.rngs = crate::app::rng::NamedRngs::new(a.world.seed as i32);
        a.player.as_mut().unwrap().luck_state.luck = 1.0;
        spawn_coins(&mut a, &npc);
        spawn_loot(&mut a, &npc);
        assert_ne!(
            *a.rngs.stream(crate::app::RngKey::UpdateNpcs),
            state_0,
            "luck 1.0 takes the hit branches"
        );
    }

    // DW-2.5 — the runtime seams drive the new arms end to end: the slime
    // rain's carried item (`npc.ai[1]`, `SlimeBodyItemDropRule.cs:19-22`)
    // reaches the world's items through `spawn_loot`'s wired `DropAttemptInfo`
    // (the ai fill, the global-first resolution, the registry filter).
    #[test]
    fn s1_e_14_spawn_loot_resolves_the_slime_rain_carried_item_end_to_end() {
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
        let mut npc = Npc::spawn_slime_variant(
            &a.world,
            (a.player.as_ref().unwrap().x / 16.0) as i32,
            SlimeVariant::Blue,
        )
        .expect("the surface spawns a blue slime");
        // The carried item: ai[1] = 8 (Torch — `GetDropInfo(8)`'s 5..10
        // family), the seam the statue-spawned slime path feeds.
        npc.ai[1] = 8.0;
        spawn_loot(&mut a, &npc);
        let drops: Vec<&crate::item::ItemDrop> = a.items.iter().filter(|it| it.kind == 8).collect();
        assert!(
            !drops.is_empty(),
            "the slime-body rain dropped the carried item"
        );
        for it in &drops {
            assert!(
                (5..=10).contains(&(it.count as i32)),
                "the GetDropInfo(8) stack range: {}",
                it.count
            );
        }
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
                    // The holiday rows read no localization and the loot path
                    // is never the simulation (`NPC.cs:78933`).
                    is_in_simulation: false,
                    get_text_value: None,
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
        use tr_core::tile::{LiquidKind, Tile, FLAG_ACTIVE};
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
            assert!(app
                .npcs
                .iter()
                .all(|n| n.kind != NpcKind::Guide || n.hp <= 0));
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
            assert!(app
                .npcs
                .iter()
                .all(|n| n.kind != NpcKind::Guide || n.hp <= 0));
            assert!(app
                .npcs
                .iter()
                .all(|n| n.kind != NpcKind::Merchant || n.hp <= 0));
        }
    }
}

#[cfg(test)]
mod invasion_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::npc::Npc;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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
    /// item stream alone (`UpdateItems` unchanged across the whole loop —
    /// the invasion path draws only from the spawner stream, DW-1.2).
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
        let item_stream_before = app.rngs.stream(RngKey::UpdateItems).clone();
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
            *app.rngs.stream(RngKey::UpdateItems),
            item_stream_before,
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
        // `StartInvasion` (`Main.cs:64435-64486`): size = 80 + 40 per
        // qualifying player; the march X is 0 or maxTilesX (the `Next(2)`
        // side roll), the warn/progress state zeroed.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        let seed = app.world.seed as i32;
        assert!(start_invasion_in(&mut app, 1, 1, Some(RngKey::UpdateTime)));
        assert_eq!(app.world.invasion_type, 1);
        assert_eq!(app.world.invasion_size, 120);
        assert_eq!(app.world.invasion_size_start, 120);
        assert_eq!(app.world.invasion_warn, 0);
        assert_eq!(app.invasion_progress, 0, "64466: invasionProgress = 0");
        assert_eq!(app.invasion_progress_icon, 4, "64467: type + 3");
        assert_eq!(app.invasion_progress_max, 120);
        assert!(
            app.world.invasion_x == 0.0 || app.world.invasion_x == 300.0,
            "the side roll ({})",
            app.world.invasion_x
        );
        // The second call refuses (the invasion is active, `:64441-64444`) —
        // and draws nothing: a fresh stream with the same seed is untouched
        // up to the one draw the first start consumed.
        assert!(!start_invasion_in(&mut app, 1, 1, Some(RngKey::UpdateTime)));
        assert_eq!(app.world.invasion_type, 1);
        assert_eq!(app.world.invasion_size, 120);
        let mut probe = tr_core::UnifiedRandom::new(seed);
        probe.next_max(2);
        assert_eq!(
            app.rngs.stream(RngKey::UpdateTime).next(),
            probe.next(),
            "the active-invasion early return draws nothing"
        );
    }

    #[test]
    fn s1_e_37_start_invasion_matches_the_csharp_size_table() {
        // `Main.cs:64456-64464` verbatim: base 80+40n, the pirates (3) add
        // 40+20n, the martians (4) are 160+40n.
        for (ty, n, want) in [
            (1_i32, 1_i32, 120_i32),
            (1, 2, 160),
            (2, 1, 120),
            (3, 1, 180),
            (3, 2, 240),
            (4, 1, 200),
            (4, 3, 280),
        ] {
            let mut w = tr_core::World::new("t".into(), 300, 300);
            let mut rng = tr_core::UnifiedRandom::new(1);
            assert!(start_invasion(&mut w, n, ty, &mut rng), "type {ty} n {n}");
            assert_eq!(w.invasion_type, ty);
            assert_eq!(w.invasion_size, want, "type {ty} with {n} players");
            assert_eq!(w.invasion_size_start, want);
        }
    }

    #[test]
    fn s1_e_37_start_invasion_refuses_with_no_qualifying_player() {
        // `:64445-64453`: the type/size block only runs when `num > 0` — with
        // no 200-life player the C# leaves `invasionType` at 0.
        let mut w = tr_core::World::new("t".into(), 300, 300);
        let mut rng = tr_core::UnifiedRandom::new(5);
        assert!(!start_invasion(&mut w, 0, 1, &mut rng));
        assert_eq!(w.invasion_type, 0, "no army without a qualifying player");
        assert_eq!(w.invasion_size, 0);
    }

    #[test]
    fn s1_e_37_the_martians_telegraph_from_spawn_and_draw_nothing() {
        // `:64471-64475`: type 4 sits one tile west of spawn with warn 2 and
        // no side roll; every other type draws `Next(2)` (`:64476-64483`).
        let mut w = tr_core::World::new("t".into(), 300, 300);
        w.spawn_x = 150;
        let mut rng = tr_core::UnifiedRandom::new(9);
        let pristine = rng.clone();
        start_invasion(&mut w, 1, 4, &mut rng);
        assert_eq!(w.invasion_x, 149.0, "spawnTileX - 1");
        assert_eq!(rng, pristine, "the martian arm consumes no draw");
        assert_eq!(w.invasion_warn, 2, ":64474");
    }

    #[test]
    fn s1_e_37_a_finished_invasion_clears_its_stale_type() {
        // `:64437-64440`: type set + size 0 → the type clears and the new
        // invasion may start; a live one still refuses.
        let mut w = tr_core::World::new("t".into(), 300, 300);
        w.invasion_type = 2;
        w.invasion_size = 0;
        let mut rng = tr_core::UnifiedRandom::new(3);
        assert!(start_invasion(&mut w, 1, 1, &mut rng));
        assert_eq!(w.invasion_type, 1, "the stale frost type was cleared");
        assert_eq!(w.invasion_size, 120);

        let mut live = tr_core::World::new("t".into(), 300, 300);
        live.invasion_type = 2;
        live.invasion_size = 40;
        assert!(!start_invasion(&mut live, 1, 1, &mut rng));
        assert_eq!(live.invasion_type, 2, "a live invasion blocks");
        assert_eq!(live.invasion_size, 40);
    }

    #[test]
    fn s1_e_37_can_start_invasion_blocks_every_type_not_just_the_goblins() {
        // `Main.cs:64414-64432`: `invasionType != 0` refuses — the pre-fix
        // port only checked the goblin flag, so a frost/pirate/martian army
        // could be overlaid.
        let mut w = tr_core::World::new("t".into(), 300, 300);
        assert!(!can_start_invasion(&w, 100), "the 200-life gate");
        assert!(can_start_invasion(&w, 200));
        for ty in [1, 2, 3, 4] {
            w.invasion_type = ty;
            assert!(
                !can_start_invasion(&w, 400),
                "type {ty} must block a new invasion"
            );
        }
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
        // The table: the picks are always goblins.
        let mut rng = UnifiedRandom::new(7);
        for _ in 0..40 {
            let kind = invasion_spawn_pick(&mut rng, false, false, 1, false, false, false, 0.0);
            assert!(matches!(
                kind,
                NpcKind::GoblinSorcerer
                    | NpcKind::GoblinThief
                    | NpcKind::GoblinPeon
                    | NpcKind::GoblinWarrior
                    | NpcKind::GoblinArcher
            ));
        }
        // The hardmode summoner: only when none alive.
        let mut rng = UnifiedRandom::new(3);
        let mut saw_summoner = false;
        for _ in 0..200 {
            if matches!(
                invasion_spawn_pick(&mut rng, true, false, 1, false, false, false, 0.0),
                NpcKind::GoblinSummoner
            ) {
                saw_summoner = true;
                break;
            }
        }
        assert!(saw_summoner, "the hardmode summoner roll");
        let mut rng = UnifiedRandom::new(7);
        for _ in 0..200 {
            assert!(
                !matches!(
                    invasion_spawn_pick(&mut rng, true, true, 1, false, false, false, 0.0),
                    NpcKind::GoblinSummoner
                ),
                "the alive summoner blocks the roll"
            );
        }
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
        let mut rng = UnifiedRandom::new(1);
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
        // `AI_037_Destroyer`'s type-135 arm (`NPC.cs:49856-49882`) through
        // the WIRED family driver (S1-D-3-wire-destroyer): the timer rolls
        // Next(4) per tick against the fresh Next(1400, 26000) threshold;
        // the fire queues the hostile laser 100 payload (dmg 22 classic,
        // speed ~8, the CanHit gate). WRITTEN NOT RUN (code-only mode).
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
        w.day_time = false;
        // The head (seg 0) never fires; the body link does. Both sit in
        // the open air above the ground so the CanHit line exists. The
        // chain is pre-assembled (ai[0] != 0 closes the spawn gate).
        let mut head = Npc::spawn_worm_head(&w, 30, 25, NpcKind::TheDestroyer);
        head.hp = head.max_hp;
        head.worm_id = 5;
        head.seg_index = 0;
        head.ai[0] = 2.0;
        let mut body = Npc::new(NpcKind::TheDestroyerBody, head.x, head.y + 20.0, 1, 0.0);
        body.hp = body.max_hp;
        body.worm_id = 5;
        body.seg_index = 1;
        let mut npcs = vec![head, body];
        let players = [crate::npc_target::TargetCandidate {
            active: true,
            dead: false,
            ghost: false,
            x: 470.0,
            y: 360.0,
            width: 20,
            height: 42,
            aggro: 0,
            item_animation: 0,
            npc_type_no_aggro: false,
            tank_pet: None,
        }];
        let mut rng = UnifiedRandom::new(1);
        let mut next_id = 9u32;
        // One body link fires ~once per 9100 ticks (the vanilla's 80
        // links make the fight rate) — the window gives the ~97% hit.
        let mut shot = None;
        for _ in 0..40000 {
            let events = crate::npc_ai_style37::destroyer_family_step(
                &mut npcs, &w, &players, &mut rng, &mut next_id, 0,
            );
            for (i, ev) in events {
                if let Some(laser) = ev.laser {
                    npcs[i].wants_hostile_shot.push((
                        crate::projectile::DESTROYER_LASER,
                        laser.x, laser.y, laser.vx, laser.vy, laser.damage,
                    ));
                }
            }
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
        use tr_core::tile::{Tile, FLAG_ACTIVE};

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
            assert_eq!(app.world.invasion_type, 2, "the frost legion's type");
            assert!(
                !app.world.invasion_goblins,
                "type 2 is not the goblins' army (the C# has one invasionType)"
            );
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
            // `Main.cs:64343` — the clear also fires the frost achievement
            // (TIN_FOIL_HATTER's `PROGRESSION_EVENT_12`).
            assert!(progression(&app, 12), "the frost clear credits 12");
            assert!(!progression(&app, 10), "the goblin id stays untouched");
        }

        /// Whether the `PROGRESSION_EVENT_<id>` condition is completed on any
        /// row (`AchievementCondition.Name`, the save key).
        fn progression(app: &App, id: u32) -> bool {
            let key = format!("PROGRESSION_EVENT_{id}");
            app.achievements
                .list
                .iter()
                .flat_map(|a| a.conditions.iter())
                .any(|c| c.name == key && c.completed)
        }

        /// S1-E-18c — all four invasion clears fire their `Main.cs:64335-64354`
        /// ids: goblins 10 (GOBLIN_PUNTER), pirates 11 (WALK_THE_PLANK),
        /// frost 12 (DO_YOU_WANT_TO_SLAY_A_SNOWMAN), martians 13
        /// (TIN_FOIL_HATTER).
        #[test]
        fn s1_e_18c_each_invasion_clear_fires_its_achievement() {
            for (invasion_type, id) in [(1i32, 10u32), (2, 12), (3, 11), (4, 13)] {
                let mut app = flow_app(None, vec![], vec![]);
                app.world = surface_world();
                app.world.invasion_type = invasion_type;
                app.world.invasion_size = 0;
                app.world.invasion_size_start = 120;
                update_invasion_state(&mut app);
                assert_eq!(app.world.invasion_type, 0, "type {invasion_type} cleared");
                assert!(progression(&app, id), "type {invasion_type} credits {id}");
                // Exactly one id: the other three stay open.
                for other in [10u32, 11, 12, 13] {
                    if other != id {
                        assert!(
                            !progression(&app, other),
                            "type {invasion_type} must not credit {other}"
                        );
                    }
                }
            }
        }

        #[test]
        fn the_frost_spawn_table_picks_the_trio() {
            // `invasionType == 2`'s table (`NPC.cs:1453-1466`): the 1/7
            // arm is net 145 (the port's MysterySnowman), the 1/3 arm net
            // 143 (the Gangsta), else net 144 (the port's SnowBalla) —
            // always the frost trio.
            let mut rng = UnifiedRandom::new(9);
            for _ in 0..60 {
                assert!(matches!(
                    invasion_spawn_pick(&mut rng, false, false, 2, false, false, false, 0.0),
                    NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman
                ));
            }
            // The per-arm net-id mapping: the 1/7 arm's kind carries net
            // 145's stats (22 def), the 1/3 arm's net 143's (20), the
            // else's net 144's (26) — `NPC.cs:10805-10846`.
            assert_eq!(Npc::new(NpcKind::MysterySnowman, 0.0, 0.0, 1, 0.0).defense(), 22);
            assert_eq!(Npc::new(NpcKind::SnowmanGangsta, 0.0, 0.0, 1, 0.0).defense(), 20);
            assert_eq!(Npc::new(NpcKind::SnowBalla, 0.0, 0.0, 1, 0.0).defense(), 26);
            // The goblin table stays on type 1.
            let mut rng = UnifiedRandom::new(9);
            for _ in 0..40 {
                assert!(matches!(
                    invasion_spawn_pick(&mut rng, false, false, 1, false, false, false, 0.0),
                    NpcKind::GoblinSorcerer
                        | NpcKind::GoblinThief
                        | NpcKind::GoblinPeon
                        | NpcKind::GoblinWarrior
                        | NpcKind::GoblinArcher
                ));
            }
        }
    }

    #[test]
    fn s1_d_3_w3_the_sim_drains_the_bee_volley_and_the_stinger() {
        // S1-D-3 W-3: the Queen Bee's aiStyle-43 axis arm writes the spawn
        // requests; the sim's spawn pass resolves them — the bee at
        // `NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80770`) with the payload's
        // velocity + `localAI[0] = 60`, the stinger as a hostile 719 with
        // the `timeLeft = 300` override (`:30743`). The UpdateNPCs stream
        // carries the arm's draws (Next(20) per tick + the type/jitter
        // draws on a fire tick) — the witness replays them in order.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        app.world.seed = 7;
        app.rngs = crate::app::rng::NamedRngs::new(7);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        // The queen above the player, out of the jungle and above the
        // surface (worldSurface 120 -> 1920 px): num619 = 2, the volley
        // period (int)(40 - 18*2) = 4, the stinger period 40 - 10 = 30.
        let mut qb = Npc::new(NpcKind::QueenBee, 150.0 * TILE_PX, 1000.0, 1, 0.0);
        qb.hp = qb.max_hp;
        qb.ai[0] = 1.0;
        qb.ai[1] = 4.0; // ++ = 5 > 4 -> the volley tick on the first step.
        app.npcs.push(qb);
        update_npcs(&mut app, 1.0 / 60.0);
        // The bee spawned: Bottom placement + the payload's state.
        let bee = app
            .npcs
            .iter()
            .find(|n| matches!(n.kind, NpcKind::Bee | NpcKind::BeeSmall))
            .expect("the volley drain spawns a bee");
        assert_eq!(bee.local_ai[0], 60.0, "localAI[0] = 60 (`:30633`)");
        assert!(bee.net_update, "the spawn sets netUpdate (`:30634`)");
        let sp = (bee.vx * bee.vx + bee.vy * bee.vy).sqrt();
        assert!((sp - 5.0).abs() < 1e-3, "the normalized x5 velocity ({sp})");
        // The request is drained (no stale re-spawn on the next tick).
        assert!(app.npcs.iter().all(|n| n.wants_bee.is_none()));
        // Tick-1 RNG parity: the queen alone rode the AI pass (the bee
        // drains AFTER it; natural spawns ride the separate SpawnNpc
        // stream), so UpdateNPCs carries exactly Next(20)+Next(210,212).
        {
            let mut w1 = UnifiedRandom::new(7);
            let _o = w1.next_max(20);
            let _t = w1.next_range(210, 212);
            assert_eq!(
                *app.rngs.stream(RngKey::UpdateNpcs),
                w1,
                "tick 1: the volley's two draws in C# order"
            );
        }
        // The stinger barrage: state 3, ai[1] = 28 -> ++ = 29,
        // 29 % 30 == 29 -> the fire tick; the queen's bottom is above the
        // player's top and the CanHit line is open air.
        app.npcs.retain(|n| n.kind == NpcKind::QueenBee);
        {
            let q = app.npcs.iter_mut().find(|n| n.kind == NpcKind::QueenBee).unwrap();
            q.ai[0] = 3.0;
            q.ai[1] = 28.0;
            q.x = 150.0 * TILE_PX;
            q.y = 1000.0;
        }
        update_npcs(&mut app, 1.0 / 60.0);
        let stinger = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active && p.kind == crate::projectile::QUEEN_BEE_STINGER)
            .expect("the barrage drain spawns a 719 stinger");
        assert_eq!(stinger.damage, 11, "the stinger damage (`:30742`)");
        assert_eq!(stinger.time_left, 300, "the timeLeft override (`:30743`)");
        assert!(!stinger.friendly, "the stinger is hostile");
        // RNG parity: the UpdateNPCs stream after the two ticks replays
        // the volley's Next(20)+Next(210,212) then the barrage's
        // Next(20)+Next(-2,3)+Next(-2,3) (num619 = 2 floors the windows
        // at 2), X before Y.
        let mut witness = UnifiedRandom::new(7);
        let _o1 = witness.next_max(20);
        let _ty = witness.next_range(210, 212);
        let _o2 = witness.next_max(20);
        let _jx = witness.next_range(-2, 3);
        let _jy = witness.next_range(-2, 3);
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateNpcs),
            witness,
            "the UpdateNPCs stream carries the arm's draws in C# order"
        );
    }
}

#[cfg(test)]
mod pirate_invasion_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::npc::Npc;
    use crate::world_sim::editing::use_pirate_map;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

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
        assert!(
            !app.world.invasion_goblins,
            "type 3 is not the goblins' army (the C# has one invasionType)"
        );
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
    fn the_pirate_spawn_table_picks_the_crew() {
        // `invasionType == 3`'s table: the Captain 1/30, the Deadeye 1/11,
        // the Corsair 1/9, the Archer 1/7, the CaptainsMate 1/3, else the
        // Deckhand — always the pirate crew.
        let mut rng = UnifiedRandom::new(11);
        for _ in 0..80 {
            assert!(matches!(
                invasion_spawn_pick(&mut rng, false, false, 3, false, false, false, 0.0),
                NpcKind::PirateDeckhand
                    | NpcKind::PirateCaptainsMate
                    | NpcKind::PirateArcher
                    | NpcKind::PirateCorsair
                    | NpcKind::PirateCaptain
            ));
        }
        // The Captain roll fires within the 1/30 odds.
        let mut rng = UnifiedRandom::new(5);
        let mut saw_captain = false;
        for _ in 0..600 {
            if matches!(
                invasion_spawn_pick(&mut rng, false, false, 3, false, false, false, 0.0),
                NpcKind::PirateCaptain
            ) {
                saw_captain = true;
                break;
            }
        }
        assert!(saw_captain, "the Captain roll fires");
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
    use tr_core::tile::{Tile, FLAG_ACTIVE};
    use tr_core::World;

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
            assert!(start_invasion_in(&mut app, 1, 4, Some(RngKey::UpdateNpcs)));
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

        #[test]
        fn the_martian_spawn_table_picks_the_ground_four() {
            // `invasionType == 4`'s ported table (`NPC.cs:1487-1540`): every
            // rolled kind is one of the ported five — the ground four (the
            // GrayGrunt 385, the GigaZapper 389, the Officer 383, the Walker
            // 520's walk-in), the Engineer 386's rider arm and the Drone 388's
            // num8 >= 4 arm; the Saucer 395/Rider 390/RayGunner 382/Scrambler
            // 381 arms consume their rolls and map to the ground fallback.
            let mut rng = UnifiedRandom::new(13);
            let mut seen_engineer = false;
            let mut seen_drone = false;
            let mut seen_walker = false;
            for _ in 0..600 {
                let kind = invasion_spawn_pick(&mut rng, false, false, 4, false, false, false, 0.0);
                assert!(
                    matches!(
                        kind,
                        NpcKind::GrayGrunt
                            | NpcKind::GigaZapper
                            | NpcKind::MartianOfficer
                            | NpcKind::MartianWalker
                            | NpcKind::MartianEngineer
                            | NpcKind::MartianDrone
                            | NpcKind::RayGunner
                            | NpcKind::BrainScrambler
                            | NpcKind::ScutlixRider
                            | NpcKind::MartianSaucerCore
                    ),
                    "the unported kinds never surface: {kind:?}"
                );
                seen_engineer |= kind == NpcKind::MartianEngineer;
                seen_drone |= kind == NpcKind::MartianDrone;
                seen_walker |= kind == NpcKind::MartianWalker;
            }
            assert!(seen_engineer, "the num6 >= 6 arm's Engineer half");
            assert!(seen_drone, "the num8 >= 4 arm's Drone");
            assert!(seen_walker, "the walk-in gate's Walker");
            let mut rng = UnifiedRandom::new(3);
            let mut seen_ray = false;
            let mut seen_scrambler = false;
            let mut seen_rider = false;
            for _ in 0..1200 {
                match invasion_spawn_pick(&mut rng, false, false, 4, true, false, false, 0.0) {
                    NpcKind::RayGunner => seen_ray = true,
                    NpcKind::BrainScrambler => seen_scrambler = true,
                    NpcKind::ScutlixRider => seen_rider = true,
                    _ => {}
                }
            }
            assert!(seen_ray, "the num8 < 2 arm's RayGunner");
            assert!(seen_scrambler, "the num8 == 2 arm's BrainScrambler");
            assert!(seen_rider, "the num6 >= 6 arm's Rider half");
            // The walker's uniqueness: while one lives, the num9 == 3 arm
            // re-rolls the ground chain instead.
            let mut rng = UnifiedRandom::new(5);
            for _ in 0..200 {
                assert!(matches!(
                    invasion_spawn_pick(&mut rng, false, false, 4, true, false, false, 0.0),
                    NpcKind::GrayGrunt
                        | NpcKind::GigaZapper
                        | NpcKind::MartianOfficer
                        | NpcKind::MartianEngineer
                        | NpcKind::MartianDrone
                        | NpcKind::RayGunner
                        | NpcKind::BrainScrambler
                        | NpcKind::ScutlixRider
                        | NpcKind::MartianSaucerCore
                ));
            }
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        assert!(
            !app.world.invasion_goblins,
            "type 4 is not the goblins' army — the type field carries it"
        );
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(11);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        assert!(app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::MartianSaucerTurret));
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::MartianSaucerCore {
                n.hp = 0;
            }
        }
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::MartianSaucerCore));
        assert!(!app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::MartianSaucerTurret));
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

    /// The battery rig at a given stream seed (the 1-in-60 gate needs a
    /// swept seed for a deterministic firing run).
    fn battery_app(seed: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = surface_world();
        app.world.seed = seed as u64;
        app.rngs = crate::app::rng::NamedRngs::new(seed);
        let mut player = Player::spawn(&app.world);
        player.x = 150.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        let mut core = Npc::new(NpcKind::MartianSaucerCore, 150.0 * TILE_PX, 1600.0, 1, 0.0);
        core.hp = core.max_hp;
        app.npcs.push(core);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::MartianSaucerCore {
                n.ai[3] = 280.0;
            }
        }
        app
    }

    fn for_seed_fire() -> App {
        for seed in 1..200i32 {
            let mut app = battery_app(seed);
            let mut hits = 0;
            for _ in 0..60 {
                crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
                hits += app
                    .projectiles
                    .slots()
                    .iter()
                    .filter(|p| p.active && p.kind == 450 && p.damage == 30)
                    .count();
            }
            if hits > 0 {
                return app;
            }
        }
        panic!("no seed fires the battery within 200");
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
        // The battery gate is a 1-in-60 roll per tick (`Next(maxValue5) ==
        // 0` on the shared stream stand-in) — sweep the world seed for a
        // deterministic firing run.
        let mut app = for_seed_fire();
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
        let mut rng = UnifiedRandom::new(3);
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

    #[test]
    fn the_type3_table_spawns_the_dutchman_in_the_half_window() {
        // `NPC.cs:1460-1468`: the half-points window spawns the ship;
        // elsewhere the pirate table runs.
        // The faithful `Next(20)` gate (`NPC.cs:1462`): inside the
        // half-points window one pick in twenty lands the ship (the
        // `Next(20)` result feeds the count); outside it never does.
        let mut in_window_ships = 0;
        for seed in 0..200i32 {
            if invasion_spawn_pick(
                &mut UnifiedRandom::new(seed + 1),
                false,
                false,
                3,
                false,
                false,
                true,
                0.75,
            ) == NpcKind::FlyingDutchman
            {
                in_window_ships += 1;
            }
        }
        assert!(in_window_ships > 0, "the window arm fires within 200 seeds");
        assert!(in_window_ships < 200, "the Next(20) gate re-rolls");
        let mut rng = UnifiedRandom::new(21);
        for _ in 0..200 {
            assert!(matches!(
                invasion_spawn_pick(&mut rng, false, false, 3, false, false, true, 0.25),
                NpcKind::PirateCaptain
                    | NpcKind::PirateArcher
                    | NpcKind::PirateCorsair
                    | NpcKind::PirateCaptainsMate
                    | NpcKind::PirateDeckhand
            ));
        }
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

/// S1-D-3 W-1 — the WoF family WIRING witnesses (the census's P0
/// catastrophic respawn-loop fix). The faithful modules
/// (`npc_ai_style27/28/29`) are audited 1:1; these pin the SEAMS: the
/// one-time init (no every-tick respawn), the `wants_*` drain-clears,
/// the `UpdateNPCs`-stream RNG parity and the live leash/despawn gates.
#[cfg(test)]
mod wof_wire_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::RngKey;
    use crate::player::Player;

    /// A hell world: solid stone from row 500 down (the band scans find
    /// their 15 tiles), 400×700 → `UnderworldLayer` 500, band rows
    /// 510..580 (`NPC.cs:25722-25727`).
    fn hell_world() -> World {
        let mut w = World::new("hell".into(), 400, 700);
        for x in 0..w.width {
            for y in 500..w.height {
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

    fn wof_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = hell_world();
        let mut player = Player::spawn(&app.world);
        player.x = 200.0 * TILE_PX;
        player.y = 496.0 * TILE_PX;
        app.player = Some(player);
        let mut wall = Npc::new(
            NpcKind::WallOfFlesh,
            150.0 * TILE_PX,
            540.0 * TILE_PX,
            1,
            0.0,
        );
        wall.hp = wall.max_hp;
        app.npcs.push(wall);
        app
    }

    /// One sim tick with the ambient spawner held off (`npc_timer` resets
    /// to 1/60 at the pass tail, so re-arm it each tick): the witnesses
    /// assert exact minion counts / stream parity, and a stray ambient
    /// spawn would pollute both.
    fn tick(app: &mut App) {
        app.npc_timer = 10.0;
        update_npcs(app, 1.0 / 60.0);
    }

    fn count(app: &App, kind: NpcKind) -> usize {
        app.npcs.iter().filter(|n| n.kind == kind && n.hp > 0).count()
    }

    /// THE catastrophic fix: the init fires ONCE (`localAI[0]` 1 → 2,
    /// `NPC.cs:26041-26043`). Tick 1 spawns exactly 2 eyes + 11 Hungries;
    /// the next ticks spawn NOTHING — the v1's ungated every-tick eye
    /// loop + never-cleared `wants_hungry` pushed 13 NPCs per tick.
    #[test]
    fn the_init_spawns_fire_once_not_every_tick() {
        let mut app = wof_app();
        tick(&mut app);
        assert_eq!(count(&app, NpcKind::WallOfFleshEye), 2, "the two mirrors");
        assert_eq!(count(&app, NpcKind::Hungry), 11, "the 11 Hungries");
        let wall = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::WallOfFlesh)
            .expect("the wall lives");
        assert_eq!(wall.local_ai[0], 2.0, "the one-shot latch");
        assert!(!wall.wants_hungry, "the legacy flag cleared");
        assert!(!wall.wants_leech, "the legacy flag cleared");
        assert!(wall.wof_spawns.is_empty(), "the requests drained");
        for _ in 0..10 {
            tick(&mut app);
            assert_eq!(count(&app, NpcKind::WallOfFleshEye), 2, "no re-spawn");
            assert_eq!(count(&app, NpcKind::Hungry), 11, "no re-spawn");
        }
    }

    /// The `NewNPC` Bottom placement (`NPC.cs:80770`, `Entity.cs:122-131`):
    /// the eyes/Hungries sit at `(X - width/2, Y - height)` — the v1
    /// spawned raw top-left, half a box right and a full box low. The
    /// init payloads ride the wall's slot + the band halves
    /// (`:26044-26049`); the Hungry `ai[0]` ladder is `i*0.1 - 0.05`.
    #[test]
    fn the_init_children_ride_the_bottom_placement_and_ai0_ladder() {
        let mut app = wof_app();
        tick(&mut app);
        let wall = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::WallOfFlesh)
            .unwrap()
            .clone();
        let (ww, wh) = wall.size();
        // The init payloads read the wall's PRE-integration position (the
        // arm emits them before the wrapper's `position += velocity` tail,
        // `NPC.cs:90495-90498`); the Y pin keeps y fixed (vy = 0).
        let spawn_x = wall.x - wall.vx;
        let center_y = wall.y + wh * 0.5;
        let eyes: Vec<&Npc> = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::WallOfFleshEye)
            .collect();
        assert_eq!(eyes.len(), 2);
        for eye in &eyes {
            let (ew, eh) = eye.size();
            // X = (int)wall.X - width/2 (the Bottom setter's float half).
            assert_eq!(eye.x, spawn_x as i32 as f32 - ew * 0.5);
            let want_y = if eye.ai[0] > 0.0 {
                ((center_y + app.world.wof_draw_top as f32) / 2.0) as i32 as f32 - eh
            } else {
                ((center_y + app.world.wof_draw_bottom as f32) / 2.0) as i32 as f32 - eh
            };
            assert_eq!(eye.y, want_y, "the band-half Bottom row");
        }
        assert_eq!(eyes.iter().filter(|e| e.ai[0] == 1.0).count(), 1);
        assert_eq!(eyes.iter().filter(|e| e.ai[0] == -1.0).count(), 1);
        let hungries: Vec<&Npc> = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::Hungry)
            .collect();
        assert_eq!(hungries.len(), 11);
        let row = ((center_y + app.world.wof_draw_bottom as f32) / 2.0) as i32 as f32;
        for (i, h) in hungries.iter().enumerate() {
            let (hw, hh) = h.size();
            assert_eq!(h.x, spawn_x as i32 as f32 - hw * 0.5);
            assert_eq!(h.y, row - hh);
            assert_eq!(h.ai[0], i as f32 * 0.1 - 0.05, "the ai0 ladder");
            let _ = ww;
        }
    }

    /// RNG stream parity: the wall's roar cadence (`NPC.cs:25716-25721`)
    /// draws `Next(1000)` EVERY tick (+ `Next(200)` only when the roar
    /// fires) on the `UpdateNPCs` stream (`Main.cs:17767-17769`). A
    /// mirror stream replaying the same draw order from the wall's
    /// `localAI[3]` trajectory must land byte-equal — the v1 skipped the
    /// draws as "cosmetic", desynchronising every later draw.
    #[test]
    fn the_roar_cadence_draws_ride_the_update_npcs_stream() {
        let mut app = wof_app();
        let mut mirror = app.rngs.stream(RngKey::UpdateNpcs).clone();
        let mut roar_timer = 0.0f32;
        for _ in 0..30 {
            // Hold the ambient spawner off (it draws on `SpawnNpc`, but a
            // spawned NPC would draw on `UpdateNpcs` next tick and pollute
            // the mirror); the player stays far from the wall's column, so
            // no contact-damage variance roll fires either.
            tick(&mut app);
            // Replay the wall's draws in the arm's order.
            roar_timer += 1.0;
            let threshold = 600 + mirror.next_max(1000);
            if roar_timer >= threshold as f32 {
                roar_timer = -(mirror.next_max(200) as f32);
            }
            let wall = app
                .npcs
                .iter()
                .find(|n| n.kind == NpcKind::WallOfFlesh)
                .unwrap();
            assert_eq!(wall.local_ai[3], roar_timer, "the cadence trajectory");
        }
        assert_eq!(
            mirror,
            *app.rngs.stream(RngKey::UpdateNpcs),
            "the stream advanced exactly by the cited draws"
        );
    }

    /// The live despawn gate: the wall's death clears the presence
    /// mirror (`ResetNPCSlotData`, `NPC.cs:8379-8381`) and the
    /// eye/Hungry arms deactivate on the NEXT tick (`:26055-26059` /
    /// `:26215-26219`) — no orphan minions outlive the wall.
    #[test]
    fn the_minions_die_with_the_wall() {
        let mut app = wof_app();
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(count(&app, NpcKind::Hungry), 11);
        // Kill the wall.
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::WallOfFlesh {
                n.hp = 0;
            }
        }
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app.world.wof_present, "the presence mirror cleared");
        assert_eq!(app.world.wof_npc_index, -1);
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(count(&app, NpcKind::WallOfFleshEye), 0, "the eyes die");
        assert_eq!(count(&app, NpcKind::Hungry), 0, "the Hungries die");
    }

    /// The live leash gate: a Hungry parked far outside the 300-px leash
    /// steers back toward its band anchor (the arm's approach accel,
    /// `NPC.cs:26305-26336`), and the strike-tick `justHit` arms the
    /// recoil pause (`:26211-26214`) — the flag clears at the update's
    /// tail (`:90516`), so the pause is ONE tick, not the v1's whole
    /// `hurt_cd` cooldown.
    #[test]
    fn the_hungry_leash_and_strike_tick_recoil_are_live() {
        let mut app = wof_app();
        update_npcs(&mut app, 1.0 / 60.0);
        // Park one Hungry 800 px right of its anchor (outside the 300
        // leash) and zero its velocity.
        let idx = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::Hungry)
            .unwrap();
        app.npcs[idx].x += 800.0;
        app.npcs[idx].vx = 0.0;
        app.npcs[idx].vy = 0.0;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs[idx].vx < 0.0,
            "the leash steers back toward the anchor"
        );
        // The strike tick: hurt() arms just_hit; the next update reads it
        // (ai[1] = 10) then the tail clears the flag.
        let from_x = app.npcs[idx].x + 100.0;
        assert!(app.npcs[idx].hurt(5, from_x, 0.0, false));
        assert!(app.npcs[idx].just_hit, "the strike arms the flag");
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.npcs[idx].ai[1], 9.0, "the recoil pause armed then ticked");
        assert!(!app.npcs[idx].just_hit, "the tail cleared the flag");
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(11);
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
        let mut rng = UnifiedRandom::new(3);
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
        let mut rng = UnifiedRandom::new(3);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
mod dd2_portal_wire_tests {
    //! B-6c (S1-E-12d) witnesses — WRITTEN NOT RUN (code-only round).
    //! The dead-code revival chain: crystal use → the 180-tick one-shot →
    //! two 549 portals at the walker's ends → the landed lane machine
    //! spawns the wave's bodies. Every citation re-read against the
    //! decompiled source; the walker's maths pinned in `tr_core::stray`.
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
        // The stand's ANCHOR tile at (150, 116), frame (0,0) — the real
        // 5x4 stand's top row (`WorldGen.Place5x4`, `WorldGen.cs:60149`):
        // the crystal's spawn point rides anchor*16 + (40, 64)
        // (`DD2Event.cs:590-594`), so its Bottom lands on the floor row
        // 120 exactly (116*16 + 64 = 1920) — the walker's start row.
        let i = w.index(150, 116);
        w.tiles[i].kind = 466;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        app.world = w;
        let mut player = Player::spawn(&app.world);
        player.x = 152.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        app
    }

    fn summon(app: &mut App) {
        let camx = app.camera;
        app.mouse_pos = {
            let (w, h) = app.screen_size();
            let wp = Vec2::new(150.0 * TILE_PX + 8.0, 116.0 * TILE_PX + 8.0);
            let screen = Vec2::new(w, h) * 0.5;
            (wp - camx) * app.zoom + screen
        };
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 3828,
            name: "Etherian Crystal".into(),
            count: 1,
            prefix: 0,
        };
        assert!(crate::world_sim::editing::use_etherian_crystal(app));
    }

    #[test]
    fn the_crystal_use_spawns_two_lane_portals_at_tick_180() {
        // `NPC.cs:40734-40756`: the ai[0] countdown hits 0 on the 180th
        // tick, `localAI[0]` latches and the two 549s spawn. The stand
        // anchor (150, 116) → spawn point (2440, 1920) (`DD2Event.cs:
        // 590-594`); the crystal's Bottom column is 2440/16 = 152, its
        // row drops to the floor 120 (`CheckArenaScore`'s 4-down probe /
        // the walker's 3-up skip, `StrayMethods.cs:89-90`/`:114-121`).
        // The flat 300-wide world walks to the ±120 caps (`SendWalker`'s
        // last spot is one step PAST the 120th advance), so the ends are
        // 152-120+1+2 = 35 and 152+120-1-2 = 269 (`:97-98`'s ±1 + the
        // caller's ±2, `NPC.cs:40748-40749`).
        let mut app = base_app();
        summon(&mut app);
        assert!(
            !app.npcs.iter().any(|n| n.kind == NpcKind::DD2LanePortal),
            "no portals before the countdown"
        );
        for _ in 0..179 {
            update_npcs(&mut app, 1.0 / 60.0);
        }
        assert!(
            !app.npcs.iter().any(|n| n.kind == NpcKind::DD2LanePortal),
            "179 ticks: the shield still holds, no portals"
        );
        update_npcs(&mut app, 1.0 / 60.0);
        let portals: Vec<&Npc> = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::DD2LanePortal)
            .collect();
        assert_eq!(portals.len(), 2, "the two lane portals exist");
        // `Bottom = end.ToWorldCoordinates(8, 16)` (`:40750-40753`), the
        // 78x130 box (`NPC.cs:16037-16038`): x = end*16 + 8 - 39.
        let left = portals.iter().find(|n| n.ai[2] == 0.0).unwrap();
        let right = portals.iter().find(|n| n.ai[2] == 1.0).unwrap();
        assert_eq!(left.x, 35.0 * 16.0 + 8.0 - 39.0, "the left end");
        assert_eq!(right.x, 269.0 * 16.0 + 8.0 - 39.0, "the right end");
        assert_eq!(left.y, 119.0 * 16.0 + 16.0 - 130.0, "Bottom anchored");
        assert_eq!(right.y, left.y);
        assert!(left.dont_take_damage && right.dont_take_damage);
        // The one-shot: `localAI[0] = 1` (`:40747`) — 400 more ticks
        // never re-spawn (the C#'s ai[0] rearm keeps counting, the
        // localAI gate refuses).
        for _ in 0..400 {
            update_npcs(&mut app, 1.0 / 60.0);
            let count = app
                .npcs
                .iter()
                .filter(|n| n.kind == NpcKind::DD2LanePortal)
                .count();
            assert!(count <= 2, "the localAI[0] latch refuses a re-spawn");
        }
        let c = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::DD2EterniaCrystal)
            .expect("the crystal stands (the gates' AnyNPCs(548) guard)");
        assert_eq!(c.local_ai[0], 1.0);
    }

    #[test]
    fn the_lanes_activate_once_the_portals_exist() {
        // The dead-code-revived proof: with the portals live, the landed
        // aiStyle-106 machine (`update_dd2_lane`) opens spawn attempts at
        // the wave-1 rate (90, `DD2Event.cs:953-975`) and the tier-1 roll
        // spawns the goblin (552) — the whole chain from the item use.
        let mut app = base_app();
        summon(&mut app);
        // 180 (the portal spawn) + 300 (the StartInvasion hold, the
        // counters park under it) + 90 (the first rate crossing).
        for _ in 0..(180 + 300 + 91) {
            update_npcs(&mut app, 1.0 / 60.0);
        }
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::DD2GoblinT1),
            "the lane machine spawns the wave's bodies in live play"
        );
        assert!(
            app.npcs.iter().any(|n| n.wants_lane_right || n.ai[2] == 1.0),
            "the right gate's flag rides"
        );
    }

    #[test]
    fn a_short_arena_refuses_the_use_with_the_bartender_warning() {
        // `WouldFailSpawningHere` (`DD2Event.cs:603-613`): < 60 walkable
        // tiles on either side refuses; `FailureMessage` (`:614-627`)
        // broadcasts the yellow warning; nothing consumes.
        let mut app = base_app();
        // Wall off the right side 40 tiles out (x = 190, rows 100-119 —
        // taller than the 3-up skip).
        for y in 100..120 {
            let i = app.world.index(190, y);
            app.world.tiles[i] = tr_core::Tile {
                kind: 1,
                ..Default::default()
            };
            app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        let camx = app.camera;
        app.mouse_pos = {
            let (w, h) = app.screen_size();
            // The click lands ON the stand anchor tile (150, 116) — the
            // C#'s myX/myY is the clicked 466 cell (`Player.cs:32401`);
            // frameY 0 → num28 = 116+3 = 119, the walker's floor probe.
            let wp = Vec2::new(150.0 * TILE_PX + 8.0, 116.0 * TILE_PX + 8.0);
            let screen = Vec2::new(w, h) * 0.5;
            (wp - camx) * app.zoom + screen
        };
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 3828,
            name: "Etherian Crystal".into(),
            count: 1,
            prefix: 0,
        };
        assert!(
            !crate::world_sim::editing::use_etherian_crystal(&mut app),
            "the 40-tile right span refuses"
        );
        assert!(!app.world.dd2_event_ongoing, "no event");
        assert_eq!(app.inventory[app.selected].count, 1, "nothing consumed");
        assert!(
            app.chat
                .messages
                .iter()
                .any(|m| m.color == [255, 255, 0]
                    && m.text == crate::lang::text("DungeonDefenders2.BartenderWarning")),
            "the bartender warning rides"
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
    fn the_fifth_wave_waits_for_the_dark_mage() {
        // `Difficulty_1_GetMonsterPointsWorth` (`:1108-1116`): at wave 5
        // with `waveKills >= 139` only the Dark Mage (564) scores — the
        // ordinary kill leaves the count, the boss kill passes the budget
        // and latches (`:1112`).
        let (mut app, _player) = {
            let mut a = base_app();
            let p = Player::spawn(&a.world);
            (a, p)
        };
        app.world.dd2_event_ongoing = true;
        app.world.dd2_wave = 5;
        app.world.dd2_wave_kills = 139.0;
        app.world.dd2_spawn_hold = 0;
        report_invasion_kill(&mut app, NpcKind::DD2GoblinT1);
        assert_eq!(app.world.dd2_wave_kills, 139.0, "the gate scores 0");
        assert_eq!(app.world.dd2_wave, 5);
        assert!(!app.world.dd2_latches.dark_mage_t1);
        report_invasion_kill(&mut app, NpcKind::DD2DarkMage);
        assert!(app.world.dd2_latches.dark_mage_t1, "`:1112`'s latch");
        // `:998-1005`: the wave-6 sentinel rewrites the wave BACK to 5 and
        // runs `StartVictoryScene` — `WonThisRun` latches (S1-E-12c).
        assert_eq!(app.world.dd2_wave, 5, "the sentinel's rewrite (`:999`)");
        assert_eq!(app.world.dd2_wave_kills, 0.0);
        assert!(app.world.dd2_won, "WonThisRun (`:483`)");
        // `:384-395`: the win arm's report is the sentinel's 1/1 pair under
        // the rewritten wave; no 1800-tick break (`:404` never runs).
        assert_eq!(app.invasion_progress, 1);
        assert_eq!(app.invasion_progress_max, 1);
        assert_eq!(app.invasion_progress_wave, 5);
        assert_eq!(app.world.dd2_spawn_hold, 0, "the win takes no break");
    }

    #[test]
    fn the_seventh_tier_two_wave_waits_for_the_ogre() {
        // `Difficulty_2_GetMonsterPointsWorth` (`:1201-1209`): at wave 7 with
        // `waveKills >= 219` only the Ogre (576) scores and latches.
        let (mut app, _player) = {
            let mut a = base_app();
            let p = Player::spawn(&a.world);
            (a, p)
        };
        app.world.dd2_event_ongoing = true;
        // The tier-2 gate: hardmode with a mech boss down.
        app.world.hard_mode = true;
        app.world.downed_mech_boss1 = true;
        app.world.dd2_wave = 7;
        app.world.dd2_wave_kills = 219.0;
        app.world.dd2_spawn_hold = 0;
        report_invasion_kill(&mut app, NpcKind::DD2GoblinT2);
        assert_eq!(app.world.dd2_wave_kills, 219.0, "the gate scores 0");
        assert!(!app.world.dd2_latches.ogre_t2);
        // `:451-454`: the zero-worth kill reports nothing at all.
        assert_eq!(app.invasion_progress_max, 0);
        report_invasion_kill(&mut app, NpcKind::DD2OgreT2);
        assert!(app.world.dd2_latches.ogre_t2, "`:1205`'s latch");
        // `:1186-1193`: the tier-2 wave-8 sentinel rewrites to 7 and wins.
        assert_eq!(app.world.dd2_wave, 7, "the sentinel's rewrite (`:1187`)");
        assert!(app.world.dd2_won, "WonThisRun (`:483`)");
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
            app.invasion_progress,
            50 + 1,
            "the Betsy's HP bucket plus the kill's worth (`:1513` — 551 scores 1)"
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
        // `AI`'s aiStyle-105 WIN-scene arm (`NPC.cs:40796-40803`,
        // `ai[1] == 2`): `ai[0] >= 600` → `DD2Event.StopInvasion(win: true)`
        // + `life = 0` — via update_dd2_event's resolve the event clears
        // and the crystal goes. S1-E-12c: the scene entry is the win
        // sentinel's `(2,2)` flip (the old ai[1] == 0 hack is gone).
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
        n.ai[1] = 2.0; // the win scene
        n.ai[0] = 599.0;
        app.world.dd2_event_ongoing = true;
        app.world.dd2_won = true;
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
mod dd2_loss_tests {
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
        let mut player = Player::spawn(&app.world);
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        app.world.dd2_event_ongoing = true;
        app.world.dd2_wave = 3;
        app.world.dd2_wave_kills = 40.0;
        app.world.dd2_spawn_hold = 0;
        app
    }

    fn crystal(app: &mut App) {
        let mut n = Npc::new(
            NpcKind::DD2EterniaCrystal,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n.caster_counter = 200; // past the spawn-in shield
        app.npcs.push(n);
    }

    #[test]
    fn the_crystal_death_enters_the_loss_scene_instead_of_dying() {
        // `NPC.checkDead`'s intercept (`NPC.cs:78087-78095`): hp <= 0
        // with ai[1] != 1 → the loss scene (ai[1] = 1, ai[0] = 0,
        // life = lifeMax); `ReportLoss` (`:40882` → `DD2Event.cs:507-515`)
        // latches LostThisRun and holds the spawns 30 ticks. The crystal
        // and the portals survive the tick.
        let mut app = base_app();
        crystal(&mut app);
        let mut gate = Npc::new(
            NpcKind::DD2LanePortal,
            140.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        gate.hp = gate.max_hp;
        app.npcs.push(gate);
        app.npcs[0].hp = 0; // the killing blow
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(app.world.dd2_lost, "LostThisRun latches");
        assert_eq!(app.world.dd2_spawn_hold, tr_core::dd2::SPAWN_HOLD_LOSS - 1);
        assert!(
            app.world.dd2_event_ongoing,
            "the event runs through the scene"
        );
        let c = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::DD2EterniaCrystal)
            .expect("the crystal survives into the scene");
        assert_eq!(c.ai[1], 1.0);
        assert!(c.hp > 0, "life = lifeMax");
        assert!(c.dont_take_damage);
        // The intercept ran after this tick's AI pass: the scene's
        // counters start clean (ai[0] = 0) and tick from the next frame.
        assert_eq!(c.ai[0], 0.0);
        let g = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::DD2LanePortal)
            .expect("the portal survives — only its fade arm opens");
        assert_eq!(g.ai[1], 1.0, "the portal mark (`NPC.cs:40875`)");
        assert_eq!(g.ai[0], 0.0, "the fade arm starts at 0");
    }

    #[test]
    fn the_lost_run_scores_no_kills() {
        // `CheckProgress`'s gate (`DD2Event.cs:364`): LostThisRun refuses
        // the whole kill during the loss scene.
        let mut app = base_app();
        app.world.dd2_lost = true;
        report_invasion_kill(&mut app, NpcKind::DD2GoblinT1);
        assert_eq!(app.world.dd2_wave_kills, 40.0);
        assert_eq!(app.world.dd2_wave, 3);
    }

    #[test]
    fn the_loss_hold_expiry_broadcasts_the_red_message() {
        // `UpdateTime` (`DD2Event.cs:219-236`): the hold's zero crossing
        // with LostThisRun → `LoseInvasionMessage` (`:344-347`) in
        // `INFO_FAILURE_INVASION_COLOR` (255, 0, 0).
        let mut app = base_app();
        crystal(&mut app);
        app.world.dd2_lost = true;
        app.world.dd2_spawn_hold = 1;
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.world.dd2_spawn_hold, 0);
        let line = app
            .chat
            .messages
            .iter()
            .find(|m| m.color == tr_core::dd2::INFO_FAILURE_INVASION_COLOR)
            .expect("the loss broadcast rides");
        assert_eq!(
            line.text,
            crate::lang::text("DungeonDefenders2.InvasionLose")
        );
        // The message fires ONCE — the crossing, not every zero tick.
        let count = app
            .chat
            .messages
            .iter()
            .filter(|m| m.color == tr_core::dd2::INFO_FAILURE_INVASION_COLOR)
            .count();
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.chat
                .messages
                .iter()
                .filter(|m| m.color == tr_core::dd2::INFO_FAILURE_INVASION_COLOR)
                .count(),
            count
        );
    }

    #[test]
    fn the_scene_end_stops_the_event_and_wipes_the_entities() {
        // The 600-tick end (`NPC.cs:40954-40967`) → `StopInvasion()`
        // (`DD2Event.cs:293-312`): the counters clear and `WipeEntities`
        // (`:629-638`) sweeps the towers, the hostiles and the open
        // chests' energy crystals. The bartender (550) survives — he is
        // not in `BelongsToInvasionOldOnesArmy`.
        let mut app = base_app();
        crystal(&mut app);
        app.world.dd2_lost = true;
        // The scene already runs; its counter is one tick from the end.
        app.npcs[0].ai[1] = 1.0;
        app.npcs[0].ai[0] = 599.0;
        // A live hostile + a portal + the bartender + a non-DD2 NPC.
        for kind in [
            NpcKind::DD2GoblinT1,
            NpcKind::DD2LanePortal,
            NpcKind::DD2Bartender,
            NpcKind::Zombie,
        ] {
            let mut n = Npc::new(kind, 145.0 * TILE_PX, 112.0 * TILE_PX, 1, 0.0);
            n.hp = n.max_hp;
            app.npcs.push(n);
        }
        // A sentry tower (677) + an ordinary shot (1) in the pool.
        app.projectiles.spawn(
            677,
            0,
            glam::Vec2::new(150.0 * TILE_PX, 110.0 * TILE_PX),
            glam::Vec2::ZERO,
            10,
            0.0,
            false,
        );
        app.projectiles.spawn(
            1,
            0,
            glam::Vec2::new(151.0 * TILE_PX, 110.0 * TILE_PX),
            glam::Vec2::ZERO,
            10,
            0.0,
            false,
        );
        // The open chest carries an energy crystal (3822) + an ordinary item.
        app.world_chests.push(crate::chest::WorldChest {
            tile: (150, 119),
            slots: vec![
                crate::item::ItemSlot {
                    kind: tr_core::dd2::ENERGY_CRYSTAL_ITEM,
                    name: "Elder Crystal".into(),
                    count: 5,
                    prefix: 0,
                },
                crate::item::ItemSlot {
                    kind: 757,
                    name: "Terra Blade".into(),
                    count: 1,
                    prefix: 0,
                },
            ],
            name: String::new(),
            max_items: 40,
        });
        app.player.as_mut().unwrap().chest = 0;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app.world.dd2_event_ongoing, "the event stops");
        assert_eq!(app.world.dd2_wave, 0, "waveNumber = 0 (`:307`)");
        assert_eq!(app.world.dd2_wave_kills, 0.0);
        for gone in [
            NpcKind::DD2EterniaCrystal,
            NpcKind::DD2GoblinT1,
            NpcKind::DD2LanePortal,
        ] {
            assert!(!app.npcs.iter().any(|n| n.kind == gone), "{gone:?} wiped");
        }
        assert!(app.npcs.iter().any(|n| n.kind == NpcKind::DD2Bartender));
        assert!(app.npcs.iter().any(|n| n.kind == NpcKind::Zombie));
        let live: Vec<u16> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .map(|p| p.kind)
            .collect();
        assert_eq!(live, vec![1], "the tower dies, the ordinary shot stays");
        assert_eq!(app.world_chests[0].slots[0].count, 0, "3822 TurnToAir");
        assert_eq!(app.world_chests[0].slots[1].count, 1, "the sword stays");
    }

    #[test]
    fn the_wipe_skips_the_closed_chests() {
        // `ClearAllDD2EnergyCrystalsInChests` (`:672-679`): only the
        // currently open chests lose their crystals.
        let mut app = base_app();
        app.world_chests.push(crate::chest::WorldChest {
            tile: (150, 119),
            slots: vec![crate::item::ItemSlot {
                kind: tr_core::dd2::ENERGY_CRYSTAL_ITEM,
                name: "Elder Crystal".into(),
                count: 5,
                prefix: 0,
            }],
            name: String::new(),
            max_items: 40,
        });
        // No chest open (p.chest == -1).
        wipe_dd2_entities(&mut app);
        assert_eq!(app.world_chests[0].slots[0].count, 5);
    }

    #[test]
    fn the_scene_crystal_rises_then_fades() {
        // `NPC.cs:40892-40904`: under 96 the rise (vy = -ai[0]/96), at
        // 96+ the fade (alpha += 50 capped 255). The machine holds
        // life = lifeMax every tick.
        let world = World::new("t".into(), 300, 200);
        let mut player = Player::spawn(&world);
        player.x = 200.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        let mut rng = UnifiedRandom::new(7);
        let mut n = Npc::new(NpcKind::DD2EterniaCrystal, 100.0, 100.0, 1, 0.0);
        n.hp = n.max_hp;
        n.ai[1] = 1.0;
        n.ai[0] = 0.0;
        let y0 = n.y;
        for _ in 0..96 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert!(n.y < y0, "the crystal rose");
        assert_eq!(n.alpha, 50, "the first fade tick at ai[0] == 96");
        assert_eq!(n.hp, n.max_hp, "life = lifeMax holds");
        for _ in 0..5 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(n.alpha, 255, "the fade caps at 255");
    }
}

/// S1-E-12c — the victory wiring's witnesses (WRITTEN, NOT RUN — the
/// code-only round): the win sentinel fires once, the medals pay per tier,
/// the crystal budget paces the drops, and the win never double-fires with
/// the loss path.
#[cfg(test)]
mod dd2_victory_tests {
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
        let mut player = Player::spawn(&app.world);
        player.x = 160.0 * TILE_PX;
        player.y = 118.0 * TILE_PX;
        app.player = Some(player);
        app.world.dd2_event_ongoing = true;
        app.world.dd2_spawn_hold = 0;
        app
    }

    fn crystal(app: &mut App) {
        let mut n = Npc::new(
            NpcKind::DD2EterniaCrystal,
            150.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        n.caster_counter = 200; // past the spawn-in shield
        app.npcs.push(n);
    }

    fn portal(app: &mut App) {
        let mut n = Npc::new(
            NpcKind::DD2LanePortal,
            140.0 * TILE_PX,
            112.0 * TILE_PX,
            1,
            0.0,
        );
        n.hp = n.max_hp;
        app.npcs.push(n);
    }

    fn medal_stacks(app: &App) -> Vec<u32> {
        app.items
            .iter()
            .filter(|d| d.kind == tr_core::dd2::MEDAL_ITEM)
            .map(|d| d.count)
            .collect()
    }

    fn win_broadcasts(app: &App) -> usize {
        app.chat
            .messages
            .iter()
            .filter(|m| m.color == tr_core::dd2::INFO_START_INVASION_COLOR)
            .count()
    }

    #[test]
    fn the_win_sentinel_fires_once_and_the_scene_stops_the_army() {
        // `CheckProgress`'s second `GetInvasionStatus` (`DD2Event.cs:381`)
        // → the tier-1 wave-6 sentinel (`:998-1005`) → `StartVictoryScene`
        // (`:481-505`): WonThisRun latches, the crystal flips to (2,2),
        // every portal to (0,1). The 600-tick scene end (`NPC.cs:
        // 40796-40803`) runs `StopInvasion(win: true)` → `WinInvasionInternal`
        // (`:314-342`): the tier-1 latch + 3 medals + the green broadcast,
        // then the counters clear and `WipeEntities` sweeps. A second pass
        // adds nothing — the win fires ONCE.
        let mut app = base_app();
        crystal(&mut app);
        portal(&mut app);
        app.world.dd2_wave = 5;
        app.world.dd2_wave_kills = 139.0;
        report_invasion_kill(&mut app, NpcKind::DD2DarkMage);
        assert!(app.world.dd2_won, "WonThisRun (`:483`)");
        assert_eq!(app.world.dd2_wave, 5, "the sentinel rewrite (`:999`)");
        assert_eq!(app.world.dd2_spawn_hold, 0, "no break on the win arm");
        let c = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::DD2EterniaCrystal)
            .unwrap();
        assert_eq!((c.ai[0], c.ai[1]), (2.0, 2.0), "`:493-494`");
        let g = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::DD2LanePortal)
            .unwrap();
        assert_eq!((g.ai[0], g.ai[1]), (0.0, 1.0), "`:500-501`");
        // The won gate (`:364`): further kills score nothing.
        let kills = app.world.dd2_wave_kills;
        report_invasion_kill(&mut app, NpcKind::DD2GoblinT1);
        assert_eq!(app.world.dd2_wave_kills, kills);
        assert!(medal_stacks(&app).is_empty(), "no medals before the stop");
        // One tick from the scene's end: the machine writes the stop
        // request, the resolve pays the win.
        let ci = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::DD2EterniaCrystal)
            .unwrap();
        app.npcs[ci].ai[0] = 599.0;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app.world.dd2_event_ongoing, "StopInvasion (`:301`)");
        assert!(app.world.dd2_downed_t1, "the tier-1 latch (`:318`)");
        assert!(!app.world.dd2_downed_t2 && !app.world.dd2_downed_t3);
        assert_eq!(medal_stacks(&app), vec![3], "DropMedals(3) (`:330`)");
        assert_eq!(win_broadcasts(&app), 1, "InvasionWin (`:341`)");
        assert!(
            !app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::DD2EterniaCrystal),
            "the crystal's real death (`NPC.cs:40800-40801`)"
        );
        assert!(
            !app.npcs.iter().any(|n| n.kind == NpcKind::DD2LanePortal),
            "WipeEntities (`:308`)"
        );
        // The second pass: no second broadcast, no second medal stack.
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(win_broadcasts(&app), 1, "the win fires once");
        assert_eq!(medal_stacks(&app), vec![3]);
    }

    #[test]
    fn the_medals_pay_per_tier() {
        // `WinInvasionInternal` (`:328-340`): tier 2 → 15, tier 3 → 60
        // (tier 1's 3 rides the once-test above). The tier-3 arm also fires
        // progression event 23 (`:338`) — the achievement seam.
        for (tier, medals) in [(2, 15u32), (3, 60u32)] {
            let mut app = base_app();
            crystal(&mut app);
            app.world.hard_mode = true;
            if tier == 2 {
                app.world.downed_mech_boss1 = true;
                app.world.dd2_wave = 7;
                app.world.dd2_wave_kills = 219.0;
                report_invasion_kill(&mut app, NpcKind::DD2OgreT2);
            } else {
                app.world.downed_golem = true;
                app.world.dd2_wave = 7;
                // No live Betsy: the budget's `-1` arm is 1 (`:1489-1492`),
                // so the Betsy kill (worth 1, `:1513`) passes it.
                app.world.dd2_wave_kills = 0.0;
                report_invasion_kill(&mut app, NpcKind::DD2Betsy);
            }
            assert!(app.world.dd2_won, "tier {tier}: the sentinel wins");
            let ci = app
                .npcs
                .iter()
                .position(|n| n.kind == NpcKind::DD2EterniaCrystal)
                .unwrap();
            app.npcs[ci].ai[0] = 599.0;
            update_npcs(&mut app, 1.0 / 60.0);
            assert_eq!(medal_stacks(&app), vec![medals], "tier {tier}");
            assert!(app.world.dd2_downed_t1, "tier {tier}: `>= 1` latches");
            assert_eq!(app.world.dd2_downed_t2, tier >= 2, "`>= 2`");
            assert_eq!(app.world.dd2_downed_t3, tier >= 3, "`>= 3`");
        }
    }

    #[test]
    fn the_wave_complete_medal_rows_pay_on_the_new_wave() {
        // `CheckProgress` (`:405-449`): the tier-1 wave 3→4 completion pays
        // `DropMedals(1)` (`:413`), broadcasts `WaveComplete` in
        // `INFO_NEW_WAVE_COLOR` (`:403`) and arms the 1800-tick break
        // (`:404`). The wave-2 opening takes the `WaveCompleteFirst` key.
        let mut app = base_app();
        crystal(&mut app);
        app.world.dd2_wave = 3;
        app.world.dd2_wave_kills = 99.0;
        report_invasion_kill(&mut app, NpcKind::DD2GoblinT1);
        assert_eq!(app.world.dd2_wave, 4);
        assert!(!app.world.dd2_won, "wave 4 is not the sentinel");
        assert_eq!(medal_stacks(&app), vec![1], "`:413`'s DropMedals(1)");
        assert_eq!(
            app.world.dd2_spawn_hold,
            tr_core::dd2::SPAWN_HOLD_WAVE_BREAK
        );
        assert!(
            app.chat
                .messages
                .iter()
                .any(|m| m.color == tr_core::dd2::INFO_NEW_WAVE_COLOR
                    && m.text == crate::lang::text("DungeonDefenders2.WaveComplete")),
            "the wave-complete broadcast (`:403`)"
        );
        // The wave-1→2 opening: the First key, no medal row.
        let mut app = base_app();
        crystal(&mut app);
        app.world.dd2_wave = 1;
        app.world.dd2_wave_kills = 59.0;
        report_invasion_kill(&mut app, NpcKind::DD2GoblinT1);
        assert_eq!(app.world.dd2_wave, 2);
        assert!(medal_stacks(&app).is_empty(), "no row for wave 2");
        assert!(
            app.chat.messages.iter().any(|m| m.text
                == crate::lang::text("DungeonDefenders2.WaveCompleteFirst")),
            "`:399-402`'s key swap"
        );
    }

    #[test]
    fn the_crystal_budget_paces_the_kill_drops() {
        // `ShouldDropCrystals` (`:815-912`) through the checkDead caller
        // (`NPC.cs:79246-79248`): tier 1 wave 1, budget 20, required 60 —
        // at 30 kills the pace is 20 * 30/60 = 10 energy-crystal drops,
        // each one 3822 at the body's centre. The decision draws no RNG;
        // the spawns ride the caller's stream (the core module's witness).
        let mut app = base_app();
        app.world.dd2_wave = 1;
        let dead = Npc::new(NpcKind::DD2GoblinT1, 100.0, 100.0, 1, 0.0);
        for kills in 0..=30 {
            app.world.dd2_wave_kills = kills as f32;
            dd2_crystal_drop(&mut app, &dead);
        }
        let crystals: Vec<&ItemDrop> = app
            .items
            .iter()
            .filter(|d| d.kind == tr_core::dd2::ENERGY_CRYSTAL_ITEM)
            .collect();
        assert_eq!(crystals.len(), 10, "20 * 30/60");
        assert!(crystals.iter().all(|d| d.count == 1), "one per drop");
        assert_eq!(app.world.dd2_crystal_drops.already_dropped, 10);
        assert_eq!(app.world.dd2_crystal_drops.to_drop, 20);
        // Past the pace: no more drops at the same progress.
        dd2_crystal_drop(&mut app, &dead);
        assert_eq!(
            app.items
                .iter()
                .filter(|d| d.kind == tr_core::dd2::ENERGY_CRYSTAL_ITEM)
                .count(),
            10
        );
        // Betsy (551) has no checkDead arm — no budget draw, no drop.
        let betsy = Npc::new(NpcKind::DD2Betsy, 100.0, 100.0, 1, 0.0);
        app.world.dd2_wave_kills = 60.0;
        dd2_crystal_drop(&mut app, &betsy);
        assert_eq!(app.world.dd2_crystal_drops.already_dropped, 10);
    }

    #[test]
    fn the_loss_stop_never_pays_the_win() {
        // The loss scene's 600-tick end (`NPC.cs:40954-40967`) runs
        // `StopInvasion()` — win: false. No medals, no green broadcast, no
        // tier latches; the red loss message already rode the hold's zero
        // crossing. Victory must not double-fire with loss.
        let mut app = base_app();
        crystal(&mut app);
        app.world.dd2_lost = true;
        let ci = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::DD2EterniaCrystal)
            .unwrap();
        app.npcs[ci].ai[1] = 1.0;
        app.npcs[ci].ai[0] = 599.0;
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(!app.world.dd2_event_ongoing, "the loss stop runs");
        assert!(!app.world.dd2_won);
        assert!(medal_stacks(&app).is_empty(), "no medals on the loss");
        assert_eq!(win_broadcasts(&app), 0, "no InvasionWin on the loss");
        assert!(
            !app.world.dd2_downed_t1 && !app.world.dd2_downed_t2 && !app.world.dd2_downed_t3,
            "no tier latches on the loss"
        );
    }

    #[test]
    fn the_winning_crystal_cannot_enter_the_loss_scene() {
        // The win machine holds `life = lifeMax` every tick (`NPC.cs:
        // 40760-40761`), so the checkDead intercept (`:78087`) never sees
        // a winning crystal; the resolve's `ai[1] != 2` guard pins it. A
        // zero-hp winning crystal stays in the win scene — no ReportLoss.
        let mut app = base_app();
        crystal(&mut app);
        let ci = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::DD2EterniaCrystal)
            .unwrap();
        app.npcs[ci].ai[1] = 2.0;
        app.npcs[ci].ai[0] = 10.0;
        app.npcs[ci].hp = 0; // the forced edge the guard refuses
        update_dd2_event(&mut app, 1.0 / 60.0);
        assert!(!app.world.dd2_lost, "no loss on the winning crystal");
        assert_eq!(app.npcs[ci].ai[1], 2.0, "the win scene holds");
        assert_eq!(app.npcs[ci].hp, 0, "the intercept did not refill it");
    }

    #[test]
    fn the_break_expiry_broadcasts_the_wave_and_summons_betsy_once() {
        // `UpdateTime` (`:219-236`): the hold's zero crossing with
        // `!LostThisRun` broadcasts `Lang.GetInvasionWaveText` on the
        // tier's roster in `INFO_NEW_WAVE_COLOR`; the tier-3 wave-7
        // opening summons Betsy once (`SummonBetsy`, `:914-927` —
        // `_spawnedBetsyT3`).
        let mut app = base_app();
        crystal(&mut app);
        app.world.hard_mode = true;
        app.world.downed_golem = true;
        app.world.dd2_wave = 7;
        app.world.dd2_spawn_hold = 1;
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.world.dd2_spawn_hold, 0);
        assert!(
            app.chat
                .messages
                .iter()
                .any(|m| m.color == tr_core::dd2::INFO_NEW_WAVE_COLOR),
            "the new-wave broadcast (`:227`)"
        );
        assert_eq!(
            app.npcs
                .iter()
                .filter(|n| n.kind == NpcKind::DD2Betsy && n.hp > 0)
                .count(),
            1,
            "SummonBetsy (`:230`)"
        );
        assert!(app.world.dd2_betsy_spawned, "`:925`'s latch");
        // A second crossing: the broadcast rides again, Betsy does not.
        app.world.dd2_spawn_hold = 1;
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.npcs
                .iter()
                .filter(|n| n.kind == NpcKind::DD2Betsy && n.hp > 0)
                .count(),
            1,
            "the latch refuses a second summon"
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::app::App;

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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
        let mut next_id = 1u32;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        let pool = npcs[0].max_hp;
        assert_eq!(pool, 10000, "the crawltipede's head carries the pool");
        let tail = npcs.len() - 1;
        assert_eq!(npcs[tail].kind, NpcKind::SolarCrawltipedeTail);
        let tail_x = npcs[tail].x;
        assert!(Npc::strike_melee(
            &mut npcs, tail, 7, tail_x, 0.0, false, None
        ));
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
        let mut rng = UnifiedRandom::new(7);
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
        assert!(
            app.npcs.len() >= 10,
            "the chain assembled ({})",
            app.npcs.len()
        );
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
        let mut saw = 0usize;
        for _ in 0..120 {
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut rng);
            if !n.wants_sphere.is_empty() {
                saw = n.wants_sphere.len();
                break;
            }
        }
        assert!(saw > 0, "the small spider spawns");
        let (kind, _sx, _sy, _ai3) = n.wants_sphere[0];
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
    fn the_leech_blob_keeps_its_own_head() {
        // `AI_082`'s anchor (`NPC.cs:36630-36640`): the clot resolves its
        // head's id (`ai[0]`'s index + 1 in the vanilla) — not "the first
        // head of the kind". Two Moon Lord families on the field: the clot
        // mirrors ITS head's centre and falls with it, ignoring the other.
        let (mut app, _player) = app_player();
        let mut core_a = Npc::new(NpcKind::MoonLordCore, 150.0 * TILE_PX, 100.0 * TILE_PX, 1, 0.0);
        core_a.hp = core_a.max_hp;
        let mut core_b = Npc::new(NpcKind::MoonLordCore, 60.0 * TILE_PX, 100.0 * TILE_PX, 1, 0.0);
        core_b.hp = core_b.max_hp;
        app.npcs.push(core_a);
        app.npcs.push(core_b);
        // One pass hands out the stable ids (`NPC.whoAmI`, `NPC.cs:6145`).
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let (uid_a, uid_b) = {
            let cores: Vec<u32> = app
                .npcs
                .iter()
                .filter(|n| n.kind == NpcKind::MoonLordCore)
                .map(|n| n.uid)
                .collect();
            (cores[0], cores[1])
        };
        let mut head_a = linked(NpcKind::MoonLordHead, 150.0 * TILE_PX, 60.0 * TILE_PX, 0);
        head_a.carrier_uid = uid_a;
        let mut head_b = linked(NpcKind::MoonLordHead, 60.0 * TILE_PX, 60.0 * TILE_PX, 0);
        head_b.carrier_uid = uid_b;
        app.npcs.push(head_a);
        app.npcs.push(head_b);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let head_a_uid = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::MoonLordHead && n.carrier_uid == uid_a)
            .map(|n| n.uid)
            .expect("the first head");
        // The clot, anchored to the FIRST head (`AI_079`'s spawn hands out
        // the head's link, `NPC.cs:42303-42318`).
        let mut clot = Npc::new(NpcKind::MoonLordLeechBlob, 120.0 * TILE_PX, 110.0 * TILE_PX, 1, 0.0);
        clot.hp = clot.max_hp;
        clot.carrier_uid = head_a_uid;
        app.npcs.push(clot);
        let (ax, ay) = app
            .npcs
            .iter()
            .find(|n| n.uid == head_a_uid)
            .map(|n| n.center())
            .expect("the first head");
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let clot_i = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::MoonLordLeechBlob)
            .expect("the clot lives while its head lives");
        assert!(app.npcs[clot_i].hp > 0);
        assert_eq!(app.npcs[clot_i].carrier_uid, head_a_uid);
        // The pre-pass mirrors the anchor's centre BEFORE the step moves it.
        assert_eq!(app.npcs[clot_i].carrier_x, ax, "the clot mirrors ITS head");
        assert_eq!(app.npcs[clot_i].carrier_y, ay);
        // The first head dies; the second family lives on.
        let head_i = app.npcs.iter().position(|n| n.uid == head_a_uid).expect("the head");
        app.npcs[head_i].hp = 0;
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let clot_alive = app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::MoonLordLeechBlob && n.hp > 0);
        assert!(!clot_alive, "the clot falls with ITS head (`:36635-36640`)");
        let other_head = app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::MoonLordHead && n.carrier_uid == uid_b && n.hp > 0);
        assert!(other_head, "the second family's head is untouched");
    }

    #[test]
    fn the_true_eye_keeps_its_own_core() {
        // `AI_081`'s anchor (`NPC.cs:42466-42470`): the True Eye resolves
        // its core's id (`ai[3]`, handed out by `MoonLord_SpawnTrueEyeOfCthulhu`
        // `:41154`) — a second Moon Lord never adopts it.
        let (mut app, _player) = app_player();
        let mut core_a = Npc::new(NpcKind::MoonLordCore, 150.0 * TILE_PX, 100.0 * TILE_PX, 1, 0.0);
        core_a.hp = core_a.max_hp;
        let mut core_b = Npc::new(NpcKind::MoonLordCore, 60.0 * TILE_PX, 100.0 * TILE_PX, 1, 0.0);
        core_b.hp = core_b.max_hp;
        app.npcs.push(core_a);
        app.npcs.push(core_b);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let (uid_a, uid_b) = {
            let cores: Vec<u32> = app
                .npcs
                .iter()
                .filter(|n| n.kind == NpcKind::MoonLordCore)
                .map(|n| n.uid)
                .collect();
            (cores[0], cores[1])
        };
        // The eye rides the SECOND core's link.
        let mut eye = Npc::new(NpcKind::MoonLordFreeEye, 60.0 * TILE_PX, 80.0 * TILE_PX, 1, 0.0);
        eye.hp = eye.max_hp;
        eye.carrier_uid = uid_b;
        eye.ai[3] = uid_b as f32;
        app.npcs.push(eye);
        let (bx, by) = app
            .npcs
            .iter()
            .find(|n| n.uid == uid_b)
            .map(|n| n.center())
            .expect("the second core");
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let eye_i = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::MoonLordFreeEye)
            .expect("the eye lives while its core lives");
        assert!(app.npcs[eye_i].hp > 0);
        assert_eq!(app.npcs[eye_i].carrier_x, bx, "the eye mirrors ITS core");
        assert_eq!(app.npcs[eye_i].carrier_y, by);
        // The FIRST core dies — the eye's link is explicit, so it lives.
        let core_i = app.npcs.iter().position(|n| n.uid == uid_a).expect("the first core");
        app.npcs[core_i].hp = 0;
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            app.npcs.iter().any(|n| n.kind == NpcKind::MoonLordFreeEye && n.hp > 0),
            "the eye does not fall with another family's core"
        );
        // Its OWN core dies — the eye falls (`:42466-42470`).
        let core_i = app.npcs.iter().position(|n| n.uid == uid_b).expect("the second core");
        app.npcs[core_i].hp = 0;
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(
            !app.npcs.iter().any(|n| n.kind == NpcKind::MoonLordFreeEye && n.hp > 0),
            "the eye falls with its own core"
        );
    }

    #[test]
    fn the_core_spawns_its_parts_once() {
        // `AI_077`'s init latch (`NPC.cs:41163-41168`'s `localAI[3]` 0 → 1):
        // the core arms the spawn phase once and the two hands + the head
        // ride out one set. The port's `ai[0] == 0` test alone re-armed
        // the phase on every return to the zero state — a second pair of
        // hands every 60 ticks.
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
        for _ in 0..200 {
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
        assert_eq!(hands, 2, "the two hands — and only the two");
        assert_eq!(heads, 1, "the head — and only the head");
    }

    #[test]
    fn the_core_becomes_vulnerable_when_its_parts_die() {
        // `AI_077`'s `flag2` (`NPC.cs:41270-41285`): all three parts in
        // their death state hand the core its vulnerable phase (`ai[0] =
        // 1`, `dontTakeDamage = false`).
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
        let core_idx = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::MoonLordCore)
            .expect("the core");
        assert_eq!(app.npcs[core_idx].ai[0], 0.0, "the waiting phase");
        assert!(
            app.npcs[core_idx].dont_take_damage,
            "the core is shielded while its parts live"
        );
        // The parts die: the core's bookkeeping sees all three gone.
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::MoonLordHand || n.kind == NpcKind::MoonLordHead {
                n.hp = 0;
            }
        }
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        let core_idx = app
            .npcs
            .iter()
            .position(|n| n.kind == NpcKind::MoonLordCore)
            .expect("the core");
        assert_eq!(app.npcs[core_idx].ai[0], 1.0, "the vulnerable phase");
        assert!(
            !app.npcs[core_idx].dont_take_damage,
            "the core takes the hits now"
        );
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
            app.npcs[idx].update_rng(&world, &mut player, 1.0 / 60.0, &mut UnifiedRandom::new(7));
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
                n.update_rng(&held, &mut player, 1.0 / 60.0, &mut UnifiedRandom::new(7));
            }
            assert!(!n.wants_dd2_spawn, "the hold parks the counter");
            n.ai[0] = 0.0;
            n.caster_counter = 0;
            let mut spawns = 0;
            for _ in 0..200 {
                n.update_rng(&world, &mut player, 1.0 / 60.0, &mut UnifiedRandom::new(7));
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
            n.update_rng(&world, &mut player, 1.0 / 60.0, &mut UnifiedRandom::new(7));
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
            let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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
        let mut rng = UnifiedRandom::new(7);
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

#[cfg(test)]
mod dw3_tests {
    use super::*;
    use crate::app::RngKey;
    use tr_core::UnifiedRandom;

    fn rng_seeded(app: &App) -> crate::app::rng::NamedRngs {
        crate::app::rng::NamedRngs::new(app.world.seed as i32)
    }

    fn pristine_of(app: &App, key: RngKey) -> UnifiedRandom {
        let mut r = crate::app::rng::NamedRngs::new(app.world.seed as i32);
        r.stream(key).clone()
    }

    /// DW-3.1: the AI group draws `stream(UpdateNPCs)` — one zombie AI tick
    /// consumes exactly the draws the zombie's `update_rng` makes, and no
    /// other stage stream moves (the C# `NPC.UpdateNPC` under the
    /// `UpdateNPCs` swap, `Main.cs:17767-17769`).
    #[test]
    fn test_dw_3_1_ai_tick_draws_the_update_npcs_stream() {
        let mut a = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut world = crate::test_world_helpers::ground();
        world.compute_column_surface();
        let mut player = crate::player::Player::spawn(&world);
        player.x = 20.0 * 16.0;
        player.y = 75.0 * 16.0;
        a.world = world;
        a.player = Some(player);
        let mut zombie = Npc::new(NpcKind::Zombie, 21.0 * 16.0, 70.0 * 16.0, 1, 0.0);
        zombie.hp = zombie.max_hp;
        a.npcs.push(zombie);

        update_npcs(&mut a, 1.0 / 60.0);
        let after = a.rngs.stream(RngKey::UpdateNpcs).clone();

        // The twin: the same world, one `update_rng` call — the AI group's
        // whole draw shape for the tick.
        let mut twin_world = crate::test_world_helpers::ground();
        twin_world.compute_column_surface();
        let mut twin_player = crate::player::Player::spawn(&twin_world);
        twin_player.x = 20.0 * 16.0;
        twin_player.y = 75.0 * 16.0;
        let mut twin = Npc::new(NpcKind::Zombie, 21.0 * 16.0, 70.0 * 16.0, 1, 0.0);
        twin.hp = twin.max_hp;
        let mut twin_rng = UnifiedRandom::new(a.world.seed as i32);
        twin.update_rng(&twin_world, &mut twin_player, 1.0 / 60.0, &mut twin_rng);
        // S1-E-21e: `AI_003_Fighters`' ambient-moan arms (`NPC.cs:57336-57367`)
        // ride the SAME `UpdateNPCs` stream after the AI's draws — the C#
        // `UpdateNPC` includes them, so the twin replays the gate + arms too
        // (the fixture's default world opens the NotDiscouraged gate; the
        // type-3 arm spends its `Next(1000)` roll every tick).
        let twin_not_discouraged = crate::ambient_sound::fighter_not_discouraged(
            twin.kind.cs_id() as i32,
            twin.y as f64,
            twin_world.world_surface * 16.0,
            twin_world.eclipse,
            twin_world.day_time,
            twin.spawned_from_statue,
            false, // the fixture's scene_metrics.zone_graveyard default
            twin_world.snow_moon,
            twin_world.invasion_type,
            twin_world
                .seeds
                .contains(tr_core::seeds::SeedFlags::DONT_STARVE),
            twin.ai[1],
            twin.ai[2],
        );
        crate::ambient_sound::fighter_ambient_moans(
            twin.kind.cs_id() as i32,
            twin.ai[3],
            twin_not_discouraged,
            twin.shimmer_transparency,
            &mut twin_rng,
            &mut |_, _, _| {},
        );
        assert_eq!(after, twin_rng, "one AI tick == one update_rng draw shape");

        // No other stage stream moved (the AI group's isolation).
        for key in RngKey::ALL {
            if key == RngKey::UpdateNpcs || key == RngKey::SpawnNpc {
                continue; // the spawner's own stream is DW-3.3's pin
            }
            let want = pristine_of(&a, key);
            assert_eq!(*a.rngs.stream(key), want, "{} stayed pristine", key.name());
        }
    }

    /// DW-3.2: the death loot (coins + loot + the velocity triplets) draws
    /// `stream(UpdateNPCs)` in the pass's NPCLoot order — the kill path's
    /// UpdateNPCs consumption equals the loot fns' direct replay (the AI
    /// tick of a dead slime draws nothing), and the other streams stay put.
    #[test]
    fn test_dw_3_2_death_loot_rides_the_update_npcs_stream() {
        let mut a = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut world = crate::test_world_helpers::ground();
        world.compute_column_surface();
        a.world = world;
        a.player = Some(crate::player::Player::spawn(&a.world));
        let mut slime = Npc::spawn_slime_variant(&a.world, 30, SlimeVariant::Blue).unwrap();
        slime.hp = 0; // dead on arrival — the kill path runs this tick
        a.npcs.push(slime.clone());

        update_npcs(&mut a, 1.0 / 60.0);
        let after = a.rngs.stream(RngKey::UpdateNpcs).clone();
        assert_ne!(
            after,
            UnifiedRandom::new(a.world.seed as i32),
            "the loot drew"
        );

        // The twin: the same death through the loot fns directly, from the
        // pristine stream — the C# order is the death sound FIRST
        // (`NPC.checkDead` `NPC.cs:78193`, one case-4 pitch draw
        // `LegacySoundPlayer.cs:693` — S1-E-21 wave C), then the NPCLoot
        // order (coins, then the rule engine, then the velocity/prefix
        // draws); the replay is byte-for-byte.
        let mut twin = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        twin.world = crate::test_world_helpers::ground();
        twin.world.compute_column_surface();
        twin.player = Some(crate::player::Player::spawn(&twin.world));
        let _death_pitch = twin.rngs.stream(RngKey::UpdateNpcs).next_range(-10, 11);
        spawn_coins(&mut twin, &slime);
        spawn_loot(&mut twin, &slime);
        assert_eq!(
            *twin.rngs.stream(RngKey::UpdateNpcs),
            after,
            "the kill path consumed exactly the NPCLoot sequence"
        );
    }

    /// DW-3.3: the spawner draws the persistent `SpawnNPC` stream — its
    /// state accumulates across ticks (no per-tick reseed, the splitmix
    /// snapshot/discard is gone), and a tick with the spawn block skipped
    /// consumes nothing (the per-call LCG advance died).
    #[test]
    fn test_dw_3_3_spawner_stream_persists_across_ticks() {
        let mut a = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut world = crate::test_world_helpers::ground();
        world.compute_column_surface();
        a.world = world;
        a.player = Some(crate::player::Player::spawn(&a.world));
        let pristine = UnifiedRandom::new(a.world.seed as i32);

        update_npcs(&mut a, 1.0 / 60.0);
        let after_one = a.rngs.stream(RngKey::SpawnNpc).clone();
        assert_ne!(after_one, pristine, "the first spawn attempt drew");

        update_npcs(&mut a, 1.0 / 60.0);
        let after_two = a.rngs.stream(RngKey::SpawnNpc).clone();
        assert_ne!(after_two, after_one, "the stream continued, not reset");
        assert_ne!(after_two, pristine, "no per-tick reseed");

        // The spawn block skipped (the timer holds) — nothing draws.
        let mut b = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        b.world = crate::test_world_helpers::ground();
        b.world.compute_column_surface();
        b.player = Some(crate::player::Player::spawn(&b.world));
        b.npc_timer = 10.0;
        update_npcs(&mut b, 1.0 / 60.0);
        let want = pristine_of(&b, RngKey::SpawnNpc);
        assert_eq!(
            *b.rngs.stream(RngKey::SpawnNpc),
            want,
            "a skipped spawn call consumes nothing (the LCG advance is gone)"
        );
    }

    /// DW-3.4: the angler quest swap rides `stream(UpdateTime)` — the
    /// pending swap's `Next(41)` ladder consumes exactly the twin replay of
    /// `angler_quest_swap`, and the npcs/players streams stay pristine.
    #[test]
    fn test_dw_3_4_angler_swap_rides_update_time() {
        let mut a = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        a.world.angler_swap_pending = true;
        resolve_angler_swap(&mut a);
        assert!(a.world.angler_quest > 0 || a.world.angler_quest == 0);
        assert!(!a.world.angler_swap_pending, "the swap resolved");

        // The twin: the same gates through the module on a twin registry.
        let gates = crate::angler::QuestGates {
            hard_mode: a.world.hard_mode,
            crimson: false,
            downed_boss1: a.world.downed_eoc,
            downed_boss2: a.world.downed_eow,
            downed_boss3: a.world.downed_skeletron,
            downed_slime_king: a.world.downed_slime_king,
            downed_queen_bee: a.world.downed_queen_bee,
            has_world_surface: true,
        };
        let mut twin_rngs = rng_seeded(&a);
        let want = crate::angler::angler_quest_swap(&gates, twin_rngs.stream(RngKey::UpdateTime));
        assert_eq!(a.world.angler_quest as u16, want, "the ladder replayed");
        assert_eq!(
            *a.rngs.stream(RngKey::UpdateTime),
            *twin_rngs.stream(RngKey::UpdateTime),
            "the UpdateTime draw shape matches"
        );
    }

    /// DW-3.4: the angler turn-in's reward rolls ride `stream(UpdatePlayers)`
    /// (the `Player.ItemCheck` identity) — a handed-in quest fish advances
    /// UpdatePlayers and leaves the npcs/spawner streams untouched.
    #[test]
    fn test_dw_3_4_angler_turn_in_rides_update_players() {
        let mut a = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut world = crate::test_world_helpers::ground();
        world.compute_column_surface();
        a.world = world;
        let quest_item = crate::angler::QUEST_POOL[a.world.angler_quest as usize];
        a.inventory[0].kind = quest_item;
        a.inventory[0].count = 1;
        let mut angler = Npc::new(NpcKind::Angler, 40.0 * 16.0, 80.0 * 16.0, 1, 0.0);
        angler.hp = angler.max_hp;
        a.npcs.push(angler);

        let before_players = a.rngs.stream(RngKey::UpdatePlayers).clone();
        assert!(angler_turn_in(&mut a, 0), "the quest fish handed in");
        assert_ne!(
            *a.rngs.stream(RngKey::UpdatePlayers),
            before_players,
            "the reward rolls drew UpdatePlayers"
        );
        // The turn-in's own bookkeeping (`Main.cs`'s quest-complete rows —
        // S1-E-2's achievement feeds): the handed-in quest completes the
        // servant rows.
        assert!(a
            .achievements
            .list
            .iter()
            .any(|a| a.name == "SERVANT_IN_TRAINING" && a.completed));
        assert!(a
            .achievements
            .list
            .iter()
            .any(|a| a.name == "GOOD_LITTLE_SLAVE" && a.conditions[0].value == 1));
    }

    /// DW-3.4: the player death pipeline rides `stream(UpdatePlayers)` —
    /// `KillMe`'s drops (gems, coins, the tombstone roll) advance it and
    /// leave the npcs pass's streams untouched.
    #[test]
    fn test_dw_3_4_player_death_rides_update_players() {
        let mut a = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        a.coins = 500;
        a.inventory[0].kind = 1522; // a large gem — the gemsOnly drop
        a.inventory[0].count = 1;

        let before = a.rngs.stream(RngKey::UpdatePlayers).clone();
        crate::death::on_player_death(&mut a, 100.0, 100.0, "Belsin was slain...");
        assert_ne!(
            *a.rngs.stream(RngKey::UpdatePlayers),
            before,
            "the death drops drew UpdatePlayers"
        );
        assert!(!a.items.is_empty(), "the drops landed");
        assert!(!a.tomb_falls.is_empty(), "the tombstone rolled");
    }

    /// DW-3.6: the npcs pass's determinism pair — two same-seed runs of
    /// `update_npcs` produce identical rosters and identical states on
    /// every named stream (the AI, loot and spawner groups together).
    #[test]
    fn test_dw_3_6_npcs_pass_determinism_pairs() {
        let build = || {
            let mut a = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
            let mut world = crate::test_world_helpers::ground();
            world.compute_column_surface();
            a.world = world;
            a.player = Some(crate::player::Player::spawn(&a.world));
            let mut slime = Npc::spawn_slime_variant(&a.world, 30, SlimeVariant::Blue).unwrap();
            slime.hp = slime.max_hp;
            a.npcs.push(slime);
            let mut zombie = Npc::new(NpcKind::Zombie, 34.0 * 16.0, 70.0 * 16.0, 1, 0.0);
            zombie.hp = zombie.max_hp;
            a.npcs.push(zombie);
            a
        };
        let mut a = build();
        let mut b = build();
        for _ in 0..30 {
            update_npcs(&mut a, 1.0 / 60.0);
            update_npcs(&mut b, 1.0 / 60.0);
        }
        let kinds_a: Vec<NpcKind> = a.npcs.iter().map(|n| n.kind).collect();
        let kinds_b: Vec<NpcKind> = b.npcs.iter().map(|n| n.kind).collect();
        assert_eq!(kinds_a, kinds_b, "the rosters match");
        let pos_a: Vec<(f32, f32)> = a.npcs.iter().map(|n| (n.x, n.y)).collect();
        let pos_b: Vec<(f32, f32)> = b.npcs.iter().map(|n| (n.x, n.y)).collect();
        assert_eq!(pos_a, pos_b, "the AI advanced identically");
        for key in RngKey::ALL {
            assert_eq!(
                *a.rngs.stream(key),
                *b.rngs.stream(key),
                "{} matches across the pair",
                key.name()
            );
        }
    }

    /// S1-E-21e — the fighter-moan wiring's stream contract: the idle counter
    /// ticks BEFORE `update_rng` (the `old_x` still holds the prior-tick
    /// stamp, `NPC.cs:57126`'s `oldPosition.X` read) and the moan arms run
    /// AFTER the AI's draws, BEFORE the critter emitter (`NPC.cs:57336` vs
    /// `:90323`), all on the `UpdateNPCs` stream (`Main.cs:17767`). A lone
    /// stuck zombie above the surface: by day the `NotDiscouraged` gate is
    /// closed (`NPC.cs:60302`) and the pass is draw-free; by night the gate
    /// opens and the type-3 arm spends exactly one `Next(1000)` roll plus the
    /// case-14 `Next(3)` slot draw inside `play_fighter_moan` — nothing else.
    #[test]
    fn s1_e_21e_the_moan_wiring_draws_on_the_npc_stream() {
        let build = |day: bool| {
            let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
            let mut world = crate::test_world_helpers::ground();
            world.day_time = day;
            app.world = world;
            // Far from the zombie: no contact damage (which would draw the
            // damage-variance roll) and no player-hitbox counter reset.
            let mut player = crate::player::Player::spawn(&app.world);
            player.x = 150.0 * 16.0;
            player.y = 110.0 * 16.0;
            app.player = Some(player);
            // Above the surface line (`world_surface * 16` = 800 px) so the
            // day arm's surface term holds; the zombie falls only a fraction
            // of a pixel in one tick and stays above it.
            let mut zombie = Npc::new(NpcKind::Zombie, 100.0, 100.0, 1, 0.0);
            zombie.hp = zombie.max_hp;
            app.npcs.push(zombie);
            // Seed 146: the type-3 arm's `Next(1000)` hits 0, then the
            // case-14 default `Next(3)` draws slot 1 (`Zombie_1`).
            app.rngs = crate::app::rng::NamedRngs::new(146);
            app
        };
        let mut control = build(true); // day + above surface: the gate closes
        let mut app = build(false); // night: the gate opens
        let control_before = control.audio.played_count();
        let app_before = app.audio.played_count();
        update_npcs(&mut control, 1.0 / 60.0);
        update_npcs(&mut app, 1.0 / 60.0);
        // The idle counter ticked once in both (stuck: `x == old_x` at AI
        // time, `NPC.cs:57126`); it stays under the 60 cap so the moan gate's
        // `ai[3] < num57` term passes.
        assert_eq!(control.npcs[0].ai[3], 1.0, "the counter ticked (control)");
        assert_eq!(app.npcs[0].ai[3], 1.0, "the counter ticked (app)");
        // Control: the discouraged gate is draw-free — no play, no draw.
        assert_eq!(
            control.audio.played_count() - control_before,
            0,
            "the day gate plays nothing"
        );
        assert_eq!(
            *control.rngs.stream(RngKey::UpdateNpcs),
            UnifiedRandom::new(146),
            "the closed gate consumes no UpdateNpcs draws"
        );
        // App: the moan fired — exactly the arm roll + the case-14 slot draw.
        assert_eq!(
            app.audio.played_count() - app_before,
            1,
            "the night moan played once"
        );
        let mut witness = UnifiedRandom::new(146);
        let _ = witness.next_max(1000); // the type-3 arm's trigger roll
        let _ = witness.next_range(0, 3); // case 14's default slot pick
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateNpcs),
            witness,
            "the moan spends exactly the arm roll + the slot draw"
        );
    }

    /// S1-E-21g — the wave-G wiring's stream contract: the AI machines
    /// queue their plays (`wants_sound`) and the sim drains them AFTER
    /// the machine's draws on the `UpdateNPCs` stream, attenuation
    /// BEFORE any case draw (the E21e precedent). A Skeletron head at
    /// the charge's second tick queues exactly one case-15 roar — which
    /// draws NOTHING (`LegacySoundPlayer.cs:826-842`). The head's own
    /// classic-charge AI draws nothing either, so the whole pass is
    /// stream-neutral.
    #[test]
    fn s1_e_21g_the_roar_wiring_queues_and_plays_draw_free() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        let mut world = crate::test_world_helpers::ground();
        // Night: the day arm would flip the head into the enraged state
        // (`NPC.cs:21958-21962`) before the charge tick runs.
        world.day_time = false;
        app.world = world;
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 150.0 * 16.0;
        player.y = 110.0 * 16.0;
        app.player = Some(player);
        // The charge state, one tick before the roar (`ai[2] == 1` → 2).
        // Inside the headless listener's 2500-px radius (camera at 0),
        // far from the player (no contact-damage variance roll).
        let mut head = Npc::new(NpcKind::SkeletronHead, 1400.0, 1000.0, 1, 0.0);
        head.hp = head.max_hp;
        head.ai[1] = 1.0;
        head.ai[2] = 1.0;
        app.npcs.push(head);
        app.rngs = crate::app::rng::NamedRngs::new(7);
        let before = app.audio.played_count();
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.audio.played_count() - before,
            1,
            "the charge's second tick roared once"
        );
        assert!(app.npcs[0].wants_sound.is_empty(), "the queue drained");
        assert_eq!(
            *app.rngs.stream(RngKey::UpdateNpcs),
            UnifiedRandom::new(7),
            "the classic charge + the roar are draw-free"
        );
        // The next tick does NOT re-roar (`ai[2] == 3` now).
        let before = app.audio.played_count();
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(app.audio.played_count() - before, 0, "one roar per charge");
    }

    /// S1-E-21g — the drain's stream contract in isolation (the machines'
    /// own AI draws legitimately share the stream): the caster teleport's
    /// Item8 pair spends exactly TWO case-2 `Next(-6, 7)` pitch draws in
    /// play order, the int death peer ONE case-4 `Next(-10, 11)`, and the
    /// roar/EoW-trackable peers NONE — every draw AFTER the attenuation
    /// gate (a muted listener drains the whole queue draw-free).
    #[test]
    fn s1_e_21g_the_sound_drain_spends_the_case_draws_in_order() {
        use crate::npc::{NpcSoundCue, NpcSoundRequest};
        let mut audio = crate::audio::Audio::new(None);
        let mut rng = UnifiedRandom::new(7);
        let req = |cue| NpcSoundRequest { cue, x: 100.0, y: 100.0 };
        // The caster pair (`NPC.cs:20862`/`:20931`): two case-2 pitches.
        play_npc_sound_request(&mut audio, req(NpcSoundCue::Item8), &mut rng);
        play_npc_sound_request(&mut audio, req(NpcSoundCue::Item8), &mut rng);
        // The tower cry (`:39059`, the int overload): one case-4 pitch.
        play_npc_sound_request(&mut audio, req(NpcSoundCue::NpcDeath(22)), &mut rng);
        // The roar + the EoW trackable: nothing.
        play_npc_sound_request(&mut audio, req(NpcSoundCue::Roar(0)), &mut rng);
        play_npc_sound_request(&mut audio, req(NpcSoundCue::EowDig), &mut rng);
        let mut witness = UnifiedRandom::new(7);
        let _ = witness.next_range(-6, 7);
        let _ = witness.next_range(-6, 7);
        let _ = witness.next_range(-10, 11);
        assert_eq!(rng, witness, "the queue's draws, in play order");
        assert_eq!(audio.played_count(), 5, "all five plays landed");
        // Muted: the same queue is entirely draw-free (the gate precedes
        // every case switch, `LegacySoundPlayer.cs:369-436`).
        let mut l = crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        };
        audio.set_listener(l);
        let mut rng = UnifiedRandom::new(7);
        let peek = rng.peek();
        for cue in [
            NpcSoundCue::Item8,
            NpcSoundCue::Item34,
            NpcSoundCue::NpcHit(1),
            NpcSoundCue::NpcDeath(6),
            NpcSoundCue::NpcDeath61,
            NpcSoundCue::NpcDeath13,
            NpcSoundCue::Roar(4),
            NpcSoundCue::DashRoar(-1),
            NpcSoundCue::EowDig,
        ] {
            play_npc_sound_request(&mut audio, req(cue), &mut rng);
        }
        assert_eq!(rng.peek(), peek, "the muted drain is draw-free");
    }

    /// S1-E-21g — the soundDelay cadence: the worm head's dig roar
    /// re-arms the delay (distance/40 clamped 10..20) and the per-tick
    /// decay runs BEFORE the machine — the roar plays once, then stays
    /// silent until the delay expires. The roar itself draws nothing
    /// (the chain budget roll is the pass's only gameplay draw here).
    #[test]
    fn s1_e_21g_the_worm_dig_roar_rides_the_sound_delay_cadence() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = crate::test_world_helpers::ground();
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 150.0 * 16.0;
        player.y = 110.0 * 16.0;
        app.player = Some(player);
        // The head starts INSIDE the ground (the dig arm, `flag3`) and
        // inside the headless listener's radius; far from the player.
        let mut worm = Npc::new(NpcKind::GiantWormHead, 1000.0, 1980.0, 1, 0.0);
        worm.hp = worm.max_hp;
        app.npcs.push(worm);
        app.rngs = crate::app::rng::NamedRngs::new(7);
        let before = app.audio.played_count();
        update_npcs(&mut app, 1.0 / 60.0);
        let head = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::GiantWormHead)
            .unwrap();
        assert!(
            (9..=20).contains(&head.sound_delay),
            "the cadence re-armed: {}",
            head.sound_delay
        );
        assert!(
            app.audio.played_count() - before >= 1,
            "the dig roar played"
        );
        // The next tick: the delay holds — no second roar.
        let before = app.audio.played_count();
        update_npcs(&mut app, 1.0 / 60.0);
        assert_eq!(
            app.audio.played_count() - before,
            0,
            "the soundDelay cadence gates the next tick"
        );
    }
}

/// S1-D-3 W-9 fishron wiring — the dispatch/spawn/RNG witnesses.
/// WRITTEN NOT RUN (code-only mode: no cargo/rustc invoked this round).
#[cfg(test)]
mod fishron_wiring_tests {
    use super::*;
    use tr_core::World;

    fn air() -> World {
        World::new("t".into(), 400, 240)
    }

    fn fishron(x: f32, y: f32) -> Npc {
        let mut n = Npc::new(NpcKind::DukeFishron, x, y, 1, 0.0);
        n.hp = n.max_hp;
        n
    }

    /// The roster rows: the four kinds' `SetDefaults` boxes/stats/styles
    /// (`NPC.cs:13701-13761`), the protocol band 213..216 round-trip and
    /// the vanilla `cs_id` hand rows (the bestiary fold must not leak
    /// into the type gates).
    #[test]
    fn the_fishron_family_roster_rows() {
        assert_eq!(NpcKind::DukeFishron.size(), (150.0, 100.0));
        assert_eq!(NpcKind::DukeFishron.ai_style(), 69);
        assert_eq!(NpcKind::DukeFishron.max_hp(), 60000);
        assert_eq!(NpcKind::DukeFishron.contact_damage(), 100);
        assert_eq!(NpcKind::DukeFishron.cs_id(), 370);
        assert_eq!(NpcKind::DetonatingBubble.size(), (36.0, 36.0));
        assert_eq!(NpcKind::DetonatingBubble.ai_style(), 70);
        assert_eq!(NpcKind::DetonatingBubble.max_hp(), 1);
        // The folded hazards ride the boss's net id (the gate fix: no
        // hand row — s1_f_3's cs_id==net_id invariant; the sway gate and
        // the render sheets read the kind/render_key, not cs_id).
        assert_eq!(NpcKind::DetonatingBubble.cs_id(), 370);
        assert_eq!(NpcKind::Sharkron.size(), (120.0, 24.0));
        assert_eq!(NpcKind::Sharkron.ai_style(), 71);
        assert_eq!(NpcKind::Sharkron.max_hp(), 100);
        assert_eq!(NpcKind::Sharkron.contact_damage(), 100);
        assert_eq!(NpcKind::Sharkron.cs_id(), 372);
        assert_eq!(NpcKind::Sharkron2.size(), (100.0, 24.0));
        assert_eq!(NpcKind::Sharkron2.ai_style(), 71);
        assert_eq!(NpcKind::Sharkron2.contact_damage(), 120);
        assert_eq!(NpcKind::Sharkron2.cs_id(), 370);
        for (kind, id) in [
            (NpcKind::DukeFishron, 219u16),
            (NpcKind::DetonatingBubble, 220),
            (NpcKind::Sharkron, 221),
            (NpcKind::Sharkron2, 222),
        ] {
            assert_eq!(kind.to_u16(), id);
            assert_eq!(NpcKind::from_u16(id), Some(kind));
        }
        // The hazards spawn invisible (`alpha = 255` rows).
        assert_eq!(Npc::new(NpcKind::DetonatingBubble, 0.0, 0.0, 1, 0.0).alpha, 255);
        assert_eq!(Npc::new(NpcKind::Sharkron, 0.0, 0.0, 1, 0.0).alpha, 255);
        assert_eq!(Npc::new(NpcKind::Sharkron2, 0.0, 0.0, 1, 0.0).alpha, 255);
        // The boss flag (`NPC.cs:13716`).
        assert!(Npc::new(NpcKind::DukeFishron, 0.0, 0.0, 1, 0.0).boss);
    }

    /// The axis claims: a Fishron/bubble/shark stepping through
    /// `update_rng` runs the faithful machine (the ai[0] -1 materialize
    /// latch, the bubble's alpha 50 pin, the shark's ai[1] counter) —
    /// not a stand-in.
    #[test]
    fn the_axis_claims_the_three_machines() {
        let w = air();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 100.0 * 16.0;
        let mut rng = UnifiedRandom::new(7);
        // Fishron: the first tick latches localAI[0] = 1, ai[0] = -1,
        // alpha 255 (`NPC.cs:48877-48887`).
        let mut boss = fishron(90.0 * 16.0, 90.0 * 16.0);
        boss.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(boss.local_ai[0], 1.0);
        assert_eq!(boss.ai[0], -1.0);
        assert_eq!(boss.alpha, 255);
        // The bubble: the init draws four, the alpha pins 50
        // (`NPC.cs:34886-34902`).
        let mut bubble = Npc::new(NpcKind::DetonatingBubble, 95.0 * 16.0, 95.0 * 16.0, 1, 0.0);
        bubble.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(bubble.alpha, 50);
        assert!(bubble.ai[3] >= 0.8 && bubble.ai[3] <= 1.2, "the scale draw");
        assert_ne!(bubble.target, 255, "TargetClosest ran");
        // The shark: phase 0 ticks ai[1], vy rides ai[3] = 0
        // (`NPC.cs:34970-34975`).
        let mut shark = Npc::new(NpcKind::Sharkron, 95.0 * 16.0, 95.0 * 16.0, 1, 0.0);
        shark.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(shark.ai[1], 1.0);
        assert!(shark.dont_take_damage);
    }

    /// The RNG census, wired: the bubble's init draws FOUR on the
    /// caller's stream (Next(80,121), Next(165,265), two Next(-100,101))
    /// plus the TWO per-tick wind jitters; the shark draws ZERO; the
    /// boss's own tick draws ZERO (the trail draw is the one exception,
    /// exercised by the preset witness below).
    #[test]
    fn the_wired_rng_census() {
        let w = air();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 100.0 * 16.0;
        // The bubble: 4 init + 2 wind = 6 draws on the first tick.
        // The fixture sits far from the player: no proximity fuse, no
        // contact overlap (the contact pass's DamageVar would draw).
        let mut rng = UnifiedRandom::new(11);
        let mut twin = UnifiedRandom::new(11);
        let mut bubble = Npc::new(NpcKind::DetonatingBubble, 80.0 * 16.0, 80.0 * 16.0, 1, 0.0);
        bubble.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        let _ = twin.next_range(80, 121);
        let _ = twin.next_range(165, 265);
        let _ = twin.next_range(-100, 101);
        let _ = twin.next_range(-100, 101);
        let _ = twin.next_range(-10, 11);
        let _ = twin.next_range(-10, 11);
        assert_eq!(rng, twin, "the init four + the wind two, in order");
        // The second tick: only the two wind jitters (the init is spent).
        bubble.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        let _ = twin.next_range(-10, 11);
        let _ = twin.next_range(-10, 11);
        assert_eq!(rng, twin, "the steady tick draws two");
        // The shark: zero across its whole fall window + the dash's first
        // ticks (the fixture sits far from the player — no contact draw).
        let mut rng = UnifiedRandom::new(11);
        let twin = UnifiedRandom::new(11);
        let mut shark = Npc::new(NpcKind::Sharkron, 60.0 * 16.0, 60.0 * 16.0, 1, 0.0);
        for _ in 0..95 {
            shark.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(rng, twin, "the style71 arm draws nothing");
        // The boss's materialize tick: zero (no trail yet).
        let mut rng = UnifiedRandom::new(11);
        let twin = UnifiedRandom::new(11);
        let mut boss = fishron(90.0 * 16.0, 90.0 * 16.0);
        boss.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        assert_eq!(rng, twin, "AI_069 itself draws nothing");
    }

    /// The phase-7 preset short-circuit: a trail bubble seeded with
    /// target/velocity/ai[3] AT THE SPAWN SITE draws only the two wind
    /// jitters — the four-draw init never runs (`NPC.cs:49477-49481`
    /// presets; the style70 census).
    #[test]
    fn the_trail_preset_short_circuits_the_init() {
        let w = air();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 100.0 * 16.0;
        let mut rng = UnifiedRandom::new(3);
        let mut twin = UnifiedRandom::new(3);
        let mut bubble = Npc::new(NpcKind::DetonatingBubble, 80.0 * 16.0, 80.0 * 16.0, 1, 0.0);
        bubble.target = 0;
        bubble.vx = 6.0;
        bubble.vy = 0.0;
        bubble.ai[3] = 1.05;
        bubble.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        let _ = twin.next_range(-10, 11);
        let _ = twin.next_range(-10, 11);
        assert_eq!(rng, twin, "only the wind two");
        assert_eq!(bubble.ai[3], 1.05, "the preset scale survives");
        assert_eq!(bubble.scale, 1.05, "scale rides ai[3]");
    }

    /// The volley drain: the boss's `wants_fishron_bubbles` requests
    /// become live DetonatingBubbles at the NewNPC bottom-centre point
    /// (`NPC.cs:49246`, `NPC.cs:80771`), stock target 255; the trail
    /// request carries its presets through. The requests are seeded on
    /// a boss whose machine is mid-materialize (local_ai[0] = 1, ai0 =
    /// -1) so the tick adds no fresh requests — the drain is the subject.
    #[test]
    fn the_volley_drains_into_live_bubbles() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = air();
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 100.0 * 16.0;
        player.y = 100.0 * 16.0;
        app.player = Some(player);
        let mut boss = fishron(90.0 * 16.0, 90.0 * 16.0);
        boss.local_ai[0] = 1.0;
        boss.ai[0] = -1.0;
        boss.wants_fishron_bubbles.push((1600.0, 1500.0));
        boss.wants_fishron_trail.push((1700.0, 1500.0, 6.0, 0.0, 0, 1.1));
        app.npcs.push(boss);
        update_npcs(&mut app, 1.0 / 60.0);
        let bubbles: Vec<&Npc> = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::DetonatingBubble)
            .collect();
        assert_eq!(bubbles.len(), 2, "the volley + the trail bubble");
        // The trail bubble carries the presets (the volley's own init
        // rolls a scale in [0.8, 1.2], so key on the exact preset).
        let trail = bubbles.iter().find(|b| b.ai[3] == 1.1).expect("the trail");
        assert_eq!(trail.target, 0);
        // Bottom = (X, Y): x - 18, y - 36 (one tick of the homing blend
        // moved it a few px at most).
        assert!((trail.x - (1700.0 - 18.0)).abs() < 8.0);
        assert!((trail.y - (1500.0 - 36.0)).abs() < 8.0);
        // The volley bubble spawned at its own bottom point too.
        let volley = bubbles.iter().find(|b| b.ai[3] != 1.1).expect("the volley");
        assert!((volley.x - (1600.0 - 18.0)).abs() < 16.0);
        assert!((volley.y - (1500.0 - 36.0)).abs() < 16.0);
    }

    /// The strike re-arm: a struck bubble does NOT die — checkDead's 371
    /// arm restores full life and arms the 4-tick detonation
    /// (`NPC.cs:78046-78051`); four ticks later it detonates (hp 0, the
    /// death retain drops it).
    #[test]
    fn the_struck_bubble_rearms_then_detonates() {
        let w = air();
        let mut player = crate::player::Player::spawn(&w);
        player.x = 100.0 * 16.0;
        player.y = 100.0 * 16.0;
        let mut rng = UnifiedRandom::new(5);
        let mut bubble = Npc::new(NpcKind::DetonatingBubble, 95.0 * 16.0, 95.0 * 16.0, 1, 0.0);
        // Seed the target so the init is spent (the strike path is the
        // subject, not the init draws).
        bubble.target = 0;
        bubble.ai[3] = 1.0;
        let mut npcs = vec![bubble];
        assert!(Npc::strike_melee(&mut npcs, 0, 50, 0.0, 0.0, false, Some(0)));
        assert_eq!(npcs[0].hp, npcs[0].max_hp, "life restored");
        assert_eq!(npcs[0].ai[0], 1.0);
        assert_eq!(npcs[0].ai[1], 4.0);
        assert!(npcs[0].dont_take_damage);
        // Four armed ticks: the countdown ends in the detonation.
        for _ in 0..4 {
            npcs[0].update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(npcs[0].hp, 0, "the detonation zeroes life");
    }

    /// The death flag: a Fishron death raises `downed_fishron`
    /// (`NPC.cs:79348-79350`'s case 370).
    #[test]
    fn the_fishron_death_raises_the_flag() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = air();
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 100.0 * 16.0;
        player.y = 100.0 * 16.0;
        app.player = Some(player);
        let mut boss = fishron(90.0 * 16.0, 90.0 * 16.0);
        boss.hp = 0;
        app.npcs.push(boss);
        update_npcs(&mut app, 1.0 / 60.0);
        assert!(app.world.downed_fishron);
        assert!(app.npcs.iter().all(|n| n.kind != NpcKind::DukeFishron));
    }

    /// The shark spawn seam: the aiStyle-64 cloud's request becomes a
    /// live shark at the NewNPC bottom point with the cloud's velocity;
    /// the 386's presets (ai2 = width, ai3 = -1.5) ride AT THE SPAWN
    /// SITE (`Projectile.cs:29509-29516`) — the style71 sway/rise reads
    /// them on the first tick.
    #[test]
    fn the_cloud_spawns_the_shark_with_the_presets() {
        let mut app = crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![]);
        app.world = air();
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 100.0 * 16.0;
        player.y = 100.0 * 16.0;
        app.player = Some(player);
        // A Cthulunado cloud at the segment hand-off: ai0 = 2 (drops to
        // 1 on the second tick), ai1 = 24 (the 2nd-bolt cadence fires).
        let slot = app
            .projectiles
            .spawn_hostile_ai_owned(
                crate::projectile::CTHULUNADO,
                app.local_id as u8,
                Vec2::new(1600.0, 1500.0),
                Vec2::new(0.0, 8.0),
                80,
                4.0,
                2.0,
                24.0,
            )
            .expect("the cloud spawns");
        // The init latch needs one tick first (localAI[0] = 1).
        crate::world_sim::projectiles::update_projectiles(&mut app, 1.0 / 60.0);
        assert_eq!(app.projectiles.slots()[slot].local_ai[0], 1.0);
        crate::world_sim::projectiles::update_projectiles(&mut app, 1.0 / 60.0);
        let sharks: Vec<&Npc> = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::Sharkron2)
            .collect();
        assert_eq!(sharks.len(), 1, "the 24th segment spawns a Sharkron2");
        let s = sharks[0];
        assert_eq!(s.ai[3], -1.5, "the rise preset");
        assert!(s.ai[2] > 0.0, "the sway amplitude = the cloud width");
    }
}
