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
        channel: app.player.is_some()
            && app.mine_down
            && {
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
