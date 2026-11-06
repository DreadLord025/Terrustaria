//! NPC simulation (plan stage A4): the fixed 60 Hz `Npc::update` step, the
//! death coin drops, the boss minion requests, the player respawn and the
//! `Spawner.SpawnNPC` ambient spawn attempt.

use glam::Vec2;
use tr_core::TILE_PX;

use crate::achievements;
use crate::app::{App, rand01};
use crate::item::ItemDrop;
use crate::npc::{Npc, NpcKind, SlimeVariant};
use crate::regen::DotDeath;
use crate::registry;
use crate::spawner;

/// The player's epitaph: a DoT drain death uses its attribution's table text
/// (`KillMe(PlayerDeathReason.ByOther(N))` — `Player.cs:19321-19331`,
/// `:19362-19385`); every other death keeps the M2-DEATH-1 stub.
fn dot_death_text(pending: Option<DotDeath>) -> &'static str {
    pending.map_or("was slain...", crate::death::death_text_for)
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
        night: !app.world.day_time,
        blood_moon: app.world.blood_moon,
        zone_snow: app.scene_metrics.zone_snow,
        zone_jungle: app.scene_metrics.zone_jungle,
        downed_boss2: false,
        downed_boss3: false,
    };
    let inv_ids: Vec<u16> = app
        .inventory
        .iter()
        .filter(|sl| sl.kind != 0 && sl.count > 0)
        .map(|sl| sl.kind)
        .collect();
    app.shop_stock = crate::shop::resolve_stock(shop, &flags, &inv_ids);
    app.shop_mood = mood.multiplier;
    app.shop_npc = Some(idx);
    // `TinkererReforge` (`NPCInteractions.cs:407-423`) opens the reforge
    // menu beside the shop (`NPCChatText_TinkererReforge`,
    // `Main.cs:40022-40029`).
    app.in_reforge_menu = app.npcs[idx].kind == NpcKind::GoblinTinkerer;
    true
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
    // The follower trail (`:51867-51922`) + the chain death
    // (`:51425-51433`).
    for i in 0..npcs.len() {
        if !npcs[i].kind.is_worm() || npcs[i].seg_index == 0 || npcs[i].hp <= 0 {
            continue;
        }
        let (wid, seg) = (npcs[i].worm_id, npcs[i].seg_index);
        let Some(leader) = npcs
            .iter()
            .find(|n| n.worm_id == wid && n.seg_index == seg - 1 && n.hp > 0)
        else {
            npcs[i].hp = 0;
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

pub(crate) fn update_npcs(app: &mut App, dt: f32) {
    app.attack_cd = (app.attack_cd - dt).max(0.0);
    // `Main.reforgeCooldown`'s per-tick decrement (`Main.cs:17121-17123`).
    if app.reforge_cooldown > 0 {
        app.reforge_cooldown -= 1;
    }
    app.npc_timer -= dt;
    // Fixed 60 Hz NPC simulation, like NPC.cs. Updating per rendered frame made
    // enemies move faster on high-refresh displays.
    let world = &app.world;
    let player = &mut app.player;
    let npcs = &mut app.npcs;
    app.npc_step.tick(dt.min(0.25), |step| {
        if let Some(player) = player {
            for n in npcs.iter_mut() {
                n.update_rng(world, player, step, &mut app.item_rng);
                black_slime_darkness_touch(n, player, &mut app.item_rng);
            }
        }
    });
    app.npc_step.cap_backlog();
    // The eye's climb-away despawn (`EncourageDespawn(10)` from the
    // aiStyle-4 day/dead arm — `EncourageDespawn` despawns once the NPC is
    // far enough off-screen; the port culls it above the player's view).
    let player_y = app.player.as_ref().map(|p| p.y).unwrap_or(0.0);
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
        for n in npcs.iter_mut() {
            if n.kind != NpcKind::Guide || n.home.is_some() {
                continue;
            }
            let tx = (n.x / TILE_PX) as i32;
            let ty = (n.y / TILE_PX) as i32;
            if let Some((hx, hy)) = crate::town::quick_find_home(world, tx, ty) {
                // `MoveTownNPC`'s success arm (`WorldGen.cs:4848-4854`):
                // the home assigns, the homeless flag clears.
                n.home = Some((hx, hy));
            }
        }
    }
    // The Goblin Army invasion (`NPC.cs:444+`, `invasionType = 1`): while
    // `invasionSize` lasts, an invader walks in near the player every 2 s
    // (the vanilla cadence rides `UpdateWorld`; the port's timer). The kind
    // mix is deterministic off the roster length (the vanilla rolls the
    // spawn RNG).
    if app.world.invasion_goblins && app.world.invasion_size > 0 {
        app.invasion_spawn_timer += dt;
        if app.invasion_spawn_timer >= 2.0 {
            app.invasion_spawn_timer = 0.0;
            if let Some(player) = app.player.as_ref() {
                let kinds = [
                    NpcKind::GoblinPeon,
                    NpcKind::GoblinPeon,
                    NpcKind::GoblinThief,
                    NpcKind::GoblinWarrior,
                    NpcKind::GoblinArcher,
                ];
                let kind = kinds[app.npcs.len() % kinds.len()];
                let dx = if app.npcs.len() % 2 == 0 { 20 } else { -20 };
                let xt = ((player.x / TILE_PX) as i32 + dx).clamp(2, app.world.width - 3);
                if let Some(n) = Npc::spawn_cave_walker(&app.world, xt, kind) {
                    app.npcs.push(n);
                }
            }
        }
    }
    // The town-NPC arrival (`NPC.SpawnAllowed_Merchant`, `NPC.cs:7175`):
    // the money condition + the absence check — the port's cadence rides
    // the 2-second town pass (the vanilla checks through UpdateWorld; the
    // dawn-gating lands with the clock events).
    if app.town_look_timer == 0.0 {
        // The Clothier: the downedSkeletron gate.
        if app.world.downed_skeletron && !app.npcs.iter().any(|n| n.kind == NpcKind::Clothier) {
            if let Some(player) = app.player.as_ref() {
                let xt = ((player.x / TILE_PX) as i32 + 11).clamp(2, app.world.width - 3);
                if let Some(mut n) = Npc::spawn_town(&app.world, xt, NpcKind::Clothier) {
                    n.home = crate::town::quick_find_home(&app.world, xt, (n.y / TILE_PX) as i32);
                    app.npcs.push(n);
                }
            }
        }
        // The Witch Doctor: the downedQueenBee gate.
        if app.world.downed_queen_bee && !app.npcs.iter().any(|n| n.kind == NpcKind::WitchDoctor) {
            if let Some(player) = app.player.as_ref() {
                let xt = ((player.x / TILE_PX) as i32 - 9).clamp(2, app.world.width - 3);
                if let Some(mut n) = Npc::spawn_town(&app.world, xt, NpcKind::WitchDoctor) {
                    n.home = crate::town::quick_find_home(&app.world, xt, (n.y / TILE_PX) as i32);
                    app.npcs.push(n);
                }
            }
        }
        // The Dryad: the downedBoss1 gate (`SpawnAllowed_Dryad`'s
        // downedBoss1 — the port's downed_eoc).
        if app.world.downed_eoc && !app.npcs.iter().any(|n| n.kind == NpcKind::Dryad) {
            if let Some(player) = app.player.as_ref() {
                let xt = ((player.x / TILE_PX) as i32 + 9).clamp(2, app.world.width - 3);
                if let Some(mut n) = Npc::spawn_town(&app.world, xt, NpcKind::Dryad) {
                    n.home = crate::town::quick_find_home(&app.world, xt, (n.y / TILE_PX) as i32);
                    app.npcs.push(n);
                }
            }
        }
        if let Some(player) = app.player.as_ref() {
            if player.max_hp > 100
                && !app.npcs.iter().any(|n| n.kind == NpcKind::Nurse)
                && !app.npcs.iter().any(|n| n.kind == NpcKind::Merchant)
            {
                // The Nurse arrives when the life maximum grew past 100
                // (`SpawnAllowed_Nurse`) — the port gates her behind the
                // Merchant's absence (the vanilla arrival picks one town
                // NPC per pass).
                let xt = ((player.x / TILE_PX) as i32 - 6).clamp(2, app.world.width - 3);
                if let Some(mut n) = Npc::spawn_town(&app.world, xt, NpcKind::Nurse) {
                    n.home = crate::town::quick_find_home(&app.world, xt, (n.y / TILE_PX) as i32);
                    app.npcs.push(n);
                }
            }
        }
    }
    if app.town_look_timer == 0.0 {
        if !app.npcs.iter().any(|n| n.kind == NpcKind::Merchant) {
            let inv: Vec<(u16, u32)> = app
                .inventory
                .iter()
                .filter(|s| s.kind != 0 && s.count > 0)
                .map(|s| (s.kind, s.count))
                .collect();
            if crate::spawner::spawn_allowed_merchant(&inv) {
                if let Some(player) = app.player.as_ref() {
                    let xt = ((player.x / TILE_PX) as i32 + 6).clamp(2, app.world.width - 3);
                    if let Some(mut m) = Npc::spawn_town(&app.world, xt, NpcKind::Merchant) {
                        m.home =
                            crate::town::quick_find_home(&app.world, xt, (m.y / TILE_PX) as i32);
                        app.npcs.push(m);
                    }
                }
            }
        }
    }
    let dead: Vec<Npc> = app.npcs.iter().filter(|n| n.hp <= 0).cloned().collect();
    app.npcs.retain(|n| n.hp > 0);
    for npc in &dead {
        // `Main.BestiaryTracker.Kills.RegisterKill` at the death's NPCLoot
        // (`NPC.cs:78835-78847`); the `GetWereThereAnyInteractions` gate is
        // constants-on — every death the port produces today is a player
        // kill, and no ported kind fails `IsNPCValidForBestiaryKillCredit`.
        let entry = crate::bestiary::entry_for_npc(npc.kind, npc.slime);
        app.kill_tracker.register_kill(entry.credit_id);
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
            NpcKind::KingSlime => app.world.downed_slime_king = true,
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
            // The invasion death bookkeeping (`NPC.VanillaKillCount`): each
            // fallen invader drains `invasionSize`; the last kill raises
            // `downedGoblins` (the Tinkerer's gate).
            NpcKind::GoblinPeon
            | NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher => {
                if app.world.invasion_goblins {
                    app.world.invasion_size -= 1;
                    if app.world.invasion_size <= 0 {
                        app.world.invasion_goblins = false;
                        app.world.downed_goblins = true;
                    }
                }
            }
            NpcKind::SkeletronHead => app.world.downed_skeletron = true,
            NpcKind::WallOfFlesh => {
                // `WorldGen.StartHardmode()` (`NPC.cs:79485`'s death hook,
                // WorldGen.cs:31707): `Main.hardMode = true` — the
                // hardmode shop rows and the ores ride the flag.
                app.world.hard_mode = true;
            }
            _ => {}
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
        if let Some((kind, sx, sy)) = n.wants_sphere.take() {
            // `NPC.NewNPC`'s `Bottom = (X, Y)` (`NPC.cs:80771`): the payload
            // is the sphere's bottom-centre.
            let (w, h) = kind.size();
            let mut ball = Npc::new(kind, sx - w * 0.5, sy - h, 1, 0.0);
            ball.hp = ball.max_hp;
            spheres.push(ball);
        }
    }
    app.npcs.append(&mut spheres);
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

    let mut minions = app.npcs.iter().filter(|n| n.kind == NpcKind::Slime).count();
    let mut spawned = Vec::new();
    for n in app.npcs.iter_mut() {
        if n.wants_minion {
            n.wants_minion = false;
            if minions < 4 {
                let xt = ((n.x / TILE_PX) as i32 + n.facing * 2).clamp(2, app.world.width - 3);
                if let Some(m) = Npc::spawn_slime(&app.world, xt) {
                    spawned.push(m);
                    minions += 1;
                }
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
            let death_text = dot_death_text(player.dot_death).to_string();
            player.dot_death = None;
            drop(player);
            // `Player.KillMe`'s chat broadcast: `{name} {text}` in the Death
            // color (`ChatColors.cs`); the ported monitor rides the App.
            crate::chat::broadcast_death(&mut app.chat, &app.player_name, &death_text);
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
                // The scene flags arrive as the systems land (hardmode
                // M5-HARDFORCE, blood moon M2-WEATHER, zones M5-BIOME); the
                // candle zones ride the M2-BUFF-5 scene scan
                // (`SceneMetrics.CalculateZones`, `:706-708`).
                flags: spawner::SpawnFlags {
                    zone_water_candle: app.scene_metrics.water_candle_count > 0,
                    zone_peace_candle: app.scene_metrics.peace_candle_count > 0,
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
            })
        });
        let mut rng = spawner::FastRandom::new(app.npc_rng);
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
                crate::prefix::roll_prefix(req.item as u16, -1, &mut app.item_rng),
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
        false, // Midas
        false, // Main.bloodMoon
        0.0,   // NPC.extraValue (master-mode bump, not modelled)
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
        // DW-1.3 — the death collection resolves the DoT table text when the
        // drain left a reason (`Player.cs:19321-19331,19362-19385`) and keeps
        // the M2-DEATH-1 stub for any other death.
        assert_eq!(dot_death_text(None), "was slain...");
        assert_eq!(dot_death_text(Some(DotDeath::Poison)), "was poisoned.");
        assert_eq!(dot_death_text(Some(DotDeath::Burned)), "was burned.");
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
