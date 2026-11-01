//! Projectile simulation (plan stage A4): the fixed 60 Hz shuriken step.

use crate::projectile::Projectile;
use crate::App;

/// Fixed 60 Hz projectile stepping. Damage lands through `Npc::hurt`, so the
/// hit cooldown and knockback are shared with melee.
pub(crate) fn update_projectiles(app: &mut App, dt: f32) {
    if app.dev_throw {
        if let Some(p) = &app.player {
            let (cx, cy) = p.center();
            app.projectiles
                .push(Projectile::shuriken(cx - 3.0, cy - 3.0, 11.0, 0.0));
        }
        app.dev_throw = false;
    }
    let world = &app.world;
    let npcs = &mut app.npcs;
    let projectiles = &mut app.projectiles;
    app.projectile_step.tick(dt.min(0.25), |step| {
        for p in projectiles.iter_mut() {
            p.update(world, npcs, step);
        }
        projectiles.retain(|p| p.alive);
    });
    app.projectile_step.cap_backlog();
}
