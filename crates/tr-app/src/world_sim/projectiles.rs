//! Projectile simulation: the fixed 60 Hz pass over the 1000-slot pool
//! (`Projectile.Update`, see `projectile.rs` for the source map).

use glam::Vec2;

use crate::app::App;
use crate::projectile::{SHURIKEN, YoyoInputs};
use crate::registry;

/// Fixed 60 Hz pool stepping. Damage lands through `Npc::hurt`, so the
/// knockback chain is shared with melee; the immunity bookkeeping is the
/// pool's (`localNPCImmunity` / `perIDStaticNPCImmunity` / `NPC.immune`).
pub(crate) fn update_projectiles(app: &mut App, dt: f32) {
    if app.dev_throw {
        if let Some(p) = &app.player {
            let (cx, cy) = p.center();
            app.projectiles.spawn(
                SHURIKEN,
                0,
                Vec2::new(cx - 3.0, cy - 3.0),
                Vec2::new(11.0, 0.0),
                10,
                12.0,
                false,
            );
        }
        app.dev_throw = false;
    }
    if app.summon_shot {
        if let Some(p) = &mut app.player {
            let (cx, cy) = p.center();
            // Phase 3's pool block is live now: the minion books against the
            // summon budget, so the debug fixture grants the slots the pair
            // needs (`maxMinions = 1` base would self-kill the Hornet on its
            // first pool tick — the cap semantics themselves are Phase 3's
            // tests; this fixture only wants both sprites on screen).
            p.max_minions = 3;
            // The DW-1.4 pair (M4-PROJ-8 Phase 1): a Hornet (373) + a Frost
            // Hydra (308) ride the generic active-slot draw pass
            // (`frame.rs`'s `p.draw` loop — the plan's Decision Log: presence
            // via the placeholder draw, not per-kind art). Damage 0 keeps the
            // PVE walk a no-op; the unported aiStyles 62/53 dispatch no arm,
            // so both slots hold position for the capture.
            // The pair anchors on the player's centre, clamped strictly
            // inside the world bounds (the guard's test is `x + w >=
            // rightWorld`, so a slot flush on the edge is out): Phase 3's
            // out-of-world guard (`Projectile.cs:16683-16703`, rightWorld /
            // bottomWorld = maxTiles * 16) would deactivate such a slot on
            // the very next pool tick, and the front-flow worlds run as
            // small as 8x8 tiles. On a real world the clamp is a no-op — the
            // DW-1.4 screenshot positions are unchanged.
            let (ww, wh) = (
                app.world.width as f32 * 16.0,
                app.world.height as f32 * 16.0,
            );
            let hx = (cx - 24.0).clamp(0.0, (ww - 24.0 - 1.0).max(0.0));
            let hy = (cy - 48.0).clamp(0.0, (wh - 26.0 - 1.0).max(0.0));
            let gx = (cx + 24.0).clamp(0.0, (ww - 80.0 - 1.0).max(0.0));
            let gy = (cy - 48.0).clamp(0.0, (wh - 74.0 - 1.0).max(0.0));
            app.projectiles.spawn_with_data_ai_owned(
                crate::projectile::HORNET,
                0,
                hx,
                hy,
                0.0,
                0.0,
                0,
                0.0,
                0.0,
                0.0,
            );
            app.projectiles.spawn_with_data_ai_owned(
                crate::projectile::FROST_HYDRA,
                0,
                gx,
                gy,
                0.0,
                0.0,
                0,
                0.0,
                0.0,
                0.0,
            );
        }
        app.summon_shot = false;
    }
    // The boomerangs' return seek reads the owner's center (`AI_003`'s
    // `Main.player[owner]` inputs).
    let owner = app.player.as_ref().map(|p| {
        let (x, y) = p.center();
        (x, y)
    });
    // `AI_099_2_Yoyos`'s `Main.player[owner]` reads: the aim world point
    // (`Main.ReverseGravitySupport(Main.MouseScreen) + Main.screenPosition`,
    // `Projectile.cs:67928` — the same mouse-world math editing.rs:26 uses),
    // the held-button channel state (`Player.channel`, the release arm
    // `:67920-67925`) and the owner's death (`:67824`).
    let (sw, sh) = app.screen_size();
    let inputs = YoyoInputs {
        aim: app.player.as_ref().map(|_| {
            (
                app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
                app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
            )
        }),
        channel: app.player.is_some() && app.mine_down && {
            let held = app.inventory[app.selected];
            held.count > 0 && registry::get(held.kind).is_some_and(|d| d.channel)
        },
        player_dead: app.player.as_ref().is_some_and(|p| p.dead),
    };
    // The closure's disjoint field captures keep the step borrow
    // (`projectile_step`) apart from the pool/world/npcs/player set. The
    // yoyo rolls (`Projectile.cs:67746`'s jitter, `:67886`'s beam rolls,
    // `:68041`'s penetrate roll) draw from the app's item stream — the
    // port's `Main.rand` analogue, the same stream the throw's crit roll
    // uses. Only this machine's projectiles simulate here — a peer's shots
    // ride the `27` wire (`Projectile.cs:17210`'s owner gate), and the
    // server-owned hostile shots (owner 255) only on the host / single-player
    // (`spawn_hostile`).
    let (local_owner, hostile_local) = (app.local_id as u8, !app.is_client());
    let mut rng = crate::world_sim::npcs::LootRng(&mut app.item_rng);
    app.projectile_step.tick(dt.min(0.25), |_| {
        app.projectiles.update(
            &mut app.world,
            &mut app.npcs,
            owner,
            app.player.as_mut(),
            inputs,
            &mut rng,
            local_owner,
            hostile_local,
        );
    });
    app.projectile_step.cap_backlog();
}

#[cfg(test)]
mod summon_fixture_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::projectile::{FROST_HYDRA, HORNET, MAX_PROJECTILES, WOODEN_ARROW};

    /// One sim tick: the player step's summon-budget reset (`Player.cs:28279-28280`
    /// — the live frame runs the player step before the pool pass) then the
    /// pool pass. Driving `update_projectiles` alone would accumulate the
    /// counters across ticks and false-trip the cap.
    fn tick(app: &mut crate::app::App) {
        if let Some(p) = &mut app.player {
            p.num_minions = 0;
            p.slots_minions = 0.0;
        }
        update_projectiles(app, 1.0 / 60.0);
    }

    /// The `TR_SUMMON` fixture arm's once-per-activation shape: the pin
    /// spawns the Hornet + Frost Hydra pair anchored on the player's center,
    /// then resets — further ticks spawn nothing and a re-armed pin spawns
    /// the second pair (the review fix: the arm had no unit test).
    #[test]
    fn the_summon_fixture_spawns_one_pair_per_activation() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        let (cx, cy) = app.player.as_ref().unwrap().center();
        app.summon_shot = true;
        tick(&mut app);
        assert!(!app.summon_shot, "the one-shot flag consumed");
        let mut hornet = None;
        let mut hydra = None;
        for p in app.projectiles.slots().iter().filter(|p| p.active) {
            match p.kind {
                HORNET => hornet = Some((p.pos.x, p.pos.y)),
                FROST_HYDRA => hydra = Some((p.pos.x, p.pos.y)),
                _ => {}
            }
        }
        // The same world-bounds clamp the fixture applies (the flow world is
        // 8x8 tiles — an unclamped 80-px Frost Hydra would fall out of world
        // and Phase 3's guard would deactivate it, exactly as the C# would).
        let (ww, wh) = (
            app.world.width as f32 * 16.0,
            app.world.height as f32 * 16.0,
        );
        // The sentry (aiStyle 53, no arm yet) holds its anchor exactly; the
        // Hornet is AI-driven since Phase 4 and drifts toward the owner on the
        // same tick — it stays anchored to within one tick's step.
        assert_eq!(
            hydra,
            Some((
                (cx + 24.0).clamp(0.0, ww - 80.0 - 1.0),
                (cy - 48.0).clamp(0.0, wh - 74.0 - 1.0),
            )),
            "the sentry anchored on the player's center (world-clamped)"
        );
        let (hx, hy) = hornet.expect("the Hornet renders");
        let anchor = (
            (cx - 24.0).clamp(0.0, ww - 24.0 - 1.0),
            (cy - 48.0).clamp(0.0, wh - 26.0 - 1.0),
        );
        assert!(
            (hx - anchor.0).abs() < 8.0 && (hy - anchor.1).abs() < 8.0,
            "the Hornet anchored within one tick's drift: {hx},{hy} vs {anchor:?}"
        );
        assert_eq!(app.projectiles.active_count(), 2, "exactly the pair");
        // The flag stays down: further ticks spawn nothing new.
        tick(&mut app);
        assert_eq!(app.projectiles.active_count(), 2, "no per-tick respawn");
        // The reset makes the arm re-armable: a second activation spawns
        // the second pair.
        app.summon_shot = true;
        tick(&mut app);
        assert!(!app.summon_shot);
        assert_eq!(app.projectiles.active_count(), 4, "the second pair");
    }

    /// A full pool declines the spawn — the C# `NewProjectile`'s 1000
    /// sentinel (`Projectile.cs:10793`) drops the request — while the arm
    /// still consumes the flag.
    #[test]
    fn the_summon_fixture_respects_a_full_pool() {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(crate::player::Player::spawn(&app.world));
        // Fill every slot with an active filler so the arm finds no space.
        for _ in 0..MAX_PROJECTILES {
            app.projectiles.spawn_with_data_ai_owned(
                WOODEN_ARROW,
                0,
                16.0,
                16.0,
                0.0,
                0.0,
                0,
                0.0,
                0.0,
                0.0,
            );
        }
        app.summon_shot = true;
        update_projectiles(&mut app, 1.0 / 60.0);
        assert!(!app.summon_shot, "the flag consumed even on the decline");
        assert_eq!(app.projectiles.active_count(), MAX_PROJECTILES);
        assert!(
            !app.projectiles
                .slots()
                .iter()
                .any(|p| p.active && (p.kind == HORNET || p.kind == FROST_HYDRA)),
            "the pair declined: the pool stayed full of fillers"
        );
    }
}
