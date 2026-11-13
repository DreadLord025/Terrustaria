//! The lunar-apocalypse chain (`WorldGen.TriggerLunarApocalypse`,
//! `WorldGen.UpdateLunarApocalypse` and `WorldGen.StartImpendingDoom`,
//! `WorldGen.cs:86626-86760`): the cultist's death raises the four towers,
//! their deaths open the Moon Lord's countdown, and the countdown spawns
//! the boss on the player nearest the world centre.

use crate::app::App;
use crate::npc::{Npc, NpcKind};
use crate::world_sim::npcs;
use tr_core::TILE_PX;

/// `WorldGen.TriggerLunarApocalypse`'s tower list (`WorldGen.cs:86628`) —
/// the four ids drawn without replacement into the four world slots.
const TOWER_ORDER: [NpcKind; 4] = [
    NpcKind::SolarTower,
    NpcKind::VortexTower,
    NpcKind::NebulaTower,
    NpcKind::StardustTower,
];

/// The countdown's constants (`NPC.cs:6035-6037`).
pub const NATURAL_COUNTDOWN: i32 = 3600;
pub const SIGIL_COUNTDOWN: i32 = 720;

/// `WorldGen.TriggerLunarApocalypse` (`WorldGen.cs:86626-86691`): the four
/// towers at `maxTilesX / 5 * (1 + j)` with the `±100` tile jitter, the
/// 30-try LOS-retry loop and the 40-tiles-above-surface fallback; all four
/// `TowerActive*` flags, `LunarApocalypseIsUp` and the four
/// `ShieldStrengthTower*` (to `ShieldStrengthTowerMax`) raise.
///
/// Deviations: the remix/no-surface branch (`:86644-86658`) is
/// constants-off (the port has no remix worlds); the retry's downward scan
/// from the surface folds into `tower_slot_clear`; the net-sync and the
/// sound are unmodelled.
pub fn trigger_lunar_apocalypse(app: &mut App, seed: u64) {
    let width = app.world.width;
    let span = width / 5;
    let shield_max = npcs::tower_shield_max(&app.world);
    // The draw-out (`:86630-86634`): each slot takes a random unused id.
    let mut pool: Vec<NpcKind> = TOWER_ORDER.to_vec();
    let mut picked = [NpcKind::SolarTower; 4];
    let mut rng = seed;
    for slot in picked.iter_mut() {
        let idx = (crate::app::rand01(&mut rng) * pool.len() as f32) as usize % pool.len();
        *slot = pool.remove(idx);
    }
    for (j, kind) in picked.iter().enumerate() {
        let center = span * (1 + j as i32);
        let mut spot: Option<(i32, i32)> = None;
        for _attempt in 0..30 {
            let jitter = (crate::app::rand01(&mut rng) * 200.0 - 100.0) as i32;
            let x = (center + jitter).clamp(2, width - 3);
            // `:86663`'s downward scan from the world surface.
            let mut y = app.world.world_surface as i32;
            while y > 100 {
                if tower_slot_clear(&app.world, x, y) {
                    break;
                }
                y -= 1;
            }
            if y > 100 {
                spot = Some((x, y));
                break;
            }
        }
        let (tx, ty) = spot.unwrap_or((center, (app.world.world_surface as i32 - 40).max(2)));
        let mut n = Npc::new(*kind, tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1, 0.0);
        n.hp = n.max_hp;
        app.npcs.push(n);
    }
    // `:86686-86688`'s flag writes.
    app.world.tower_active_solar = true;
    app.world.tower_active_vortex = true;
    app.world.tower_active_nebula = true;
    app.world.tower_active_stardust = true;
    app.world.lunar_apocalypse_is_up = true;
    app.world.tower_shield_solar = shield_max;
    app.world.tower_shield_vortex = shield_max;
    app.world.tower_shield_nebula = shield_max;
    app.world.tower_shield_stardust = shield_max;
}

/// `WorldGen.MessageLunarApocalypse` (`WorldGen.cs:86778-86801`): the
/// inactive-tower count picks the `Lang.misc[43 + num]` line — the port
/// carries the four message strings.
pub fn message_lunar_apocalypse(app: &mut App) {
    let w = &app.world;
    if !w.lunar_apocalypse_is_up {
        return;
    }
    let mut down = 0;
    for active in [
        w.tower_active_solar,
        w.tower_active_vortex,
        w.tower_active_nebula,
        w.tower_active_stardust,
    ] {
        if !active {
            down += 1;
        }
    }
    let text = match down {
        0 => "The celestial towers descend upon the land...",
        1 => "One tower stands no more.",
        2 => "Half the towers have fallen.",
        3 => "One tower remains.",
        _ => "The last tower wavers.",
    };
    crate::chat::broadcast_death_text(&mut app.chat, text, crate::chat::COLOR_COMMAND);
}

/// `WorldGen.UpdateLunarApocalypse` (`WorldGen.cs:86693-86748`): clear the
/// `TowerActive*` flags for the towers that died, and when all four are
/// down (and the boss isn't up) `StartImpendingDoom(3600)`.
pub fn update_lunar_apocalypse(app: &mut App) {
    if !app.world.lunar_apocalypse_is_up {
        return;
    }
    let mut alive = [false; 5];
    for n in app.npcs.iter() {
        if n.hp <= 0 {
            continue;
        }
        let slot = match n.kind {
            NpcKind::SolarTower => 1,
            NpcKind::NebulaTower => 2,
            NpcKind::NebulaTower => 3,
            NpcKind::StardustTower => 4,
            NpcKind::MoonLordCore
            | NpcKind::MoonLordHead
            | NpcKind::MoonLordHand
            | NpcKind::MoonLordFreeEye => 0,
            _ => continue,
        };
        alive[slot] = true;
    }
    // `:86728-86743`'s per-flag clears.
    app.world.tower_active_solar = alive[1];
    app.world.tower_active_vortex = alive[2];
    app.world.tower_active_nebula = alive[3];
    app.world.tower_active_stardust = alive[4];
    if !alive[1] && !alive[2] && !alive[3] && !alive[4] && !alive[0] {
        start_impending_doom(app, NATURAL_COUNTDOWN);
    }
}

/// `WorldGen.StartImpendingDoom` (`WorldGen.cs:86750-86760`): the apocalypse
/// ends and the countdown arms.
///
/// Deviations: `GetRidOfCultists` (`:86763-86776`) targets the ritual kinds
/// 437/438/439, which the port does not carry (its cultist is the
/// tower-shaped `AncientCultist`) — the arm is constants-off; the net
/// packet 103 is unmodelled.
pub fn start_impending_doom(app: &mut App, countdown: i32) {
    app.world.lunar_apocalypse_is_up = false;
    app.world.moon_lord_countdown = countdown;
    app.world.max_moon_lord_countdown = countdown;
    crate::chat::broadcast_death_text(
        &mut app.chat,
        "The air is growing colder...",
        crate::chat::COLOR_COMMAND,
    );
}

/// `Main.UpdateTime`'s Moon-Lord block (`Main.cs:64971-64994`): the
/// countdown ticks down and at zero spawns the boss on the player closest
/// to `(maxTilesX / 2, worldSurface / 2) * 16` — the port's single player
/// stands in for `Player.FindClosest`.
pub fn update_moon_lord_countdown(app: &mut App) {
    // S1-G-6: the MoonLordShake filter driver (`Main.cs:64975-64984`) runs
    // BEFORE the decrement, on the pre-decrement value, under `!Main.dedServ`
    // (the port is never a dedicated server). `intensity = clamp(sin((float)
    // countdown / 60f * 0.5f) * 2f, 0f, 1f) * (0.75f - 0.5f * ((float)
    // countdown / (float)max))` — the `Math.Sin` argument is float math and
    // the sine runs in double, cast back (`:64977-64979`). `UseIntensity`
    // persists while the filter is inactive; the activation gate follows the
    // countdown.
    if app.world.moon_lord_countdown > 0 {
        let cd = app.world.moon_lord_countdown as f32;
        let max = app.world.max_moon_lord_countdown as f32;
        let arg = cd / 60.0f32 * 0.5f32;
        let intensity =
            ((arg as f64).sin() as f32 * 2.0f32).clamp(0.0, 1.0) * (0.75f32 - 0.5f32 * (cd / max));
        app.filter_driver.moon_lord_shake_intensity = Some(intensity);
        app.filter_driver.moon_lord_shake_wanted = true;
    } else {
        app.filter_driver.moon_lord_shake_wanted = false;
    }
    if app.world.moon_lord_countdown <= 0 {
        return;
    }
    app.world.moon_lord_countdown -= 1;
    if app.world.moon_lord_countdown > 0 {
        return;
    }
    let (px, py) = app.player.as_ref().map(|p| p.center()).unwrap_or((
        app.world.width as f32 * TILE_PX * 0.5,
        app.world.world_surface as f32 * TILE_PX,
    ));
    let mut core = Npc::new(NpcKind::MoonLordCore, px, py - 150.0, 1, 0.0);
    core.hp = core.max_hp;
    core.ai[0] = -1.0; // `AI_077`'s spawn-in phase.
    app.npcs.push(core);
}

/// `NPC.AnyoneNearCultists` (`NPC.cs:80290-80313`): the 2500×1500
/// player-rect test around the cultist — the port reads the
/// `AncientCultist`'s live rect against the player's.
pub fn anyone_near_cultists(app: &App) -> bool {
    let Some((px, py)) = app.player.as_ref().map(|p| p.center()) else {
        return false;
    };
    let Some(c) = app
        .npcs
        .iter()
        .find(|n| n.kind == NpcKind::AncientCultist && n.hp > 0)
    else {
        return false;
    };
    let (ccx, ccy) = c.center();
    // `Utils.CenteredRectangle(Center, (2500, 1500))`: the half-extents are
    // 1250 × 750 around the cultist's centre.
    (px - ccx).abs() < 1250.0 && (py - ccy).abs() < 750.0
}

/// `WorldGen.cs:86663`'s slot test: the tower's 20-tile vertical and
/// 10-tile horizontal clearance (`!Collision.SolidTiles(x-10, x+10, y-20,
/// y+15)`) plus the four `PlayerLOS` probes — the port's `tile()` lookup
/// stands in for the solid scan (the LOS probes fold in: the slot is clear
/// only when no solid tile blocks it).
fn tower_slot_clear(world: &tr_core::World, tx: i32, ty: i32) -> bool {
    for bx in (tx - 10)..=(tx + 10) {
        for by in (ty - 20)..=(ty + 15) {
            if !(0..world.width).contains(&bx) || !(0..world.height).contains(&by) {
                continue;
            }
            if world
                .tile(bx, by)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
            {
                return false;
            }
        }
    }
    true
}
