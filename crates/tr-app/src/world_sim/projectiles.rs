//! Projectile simulation: the fixed 60 Hz pass over the 1000-slot pool
//! (`Projectile.Update`, see `projectile.rs` for the source map).

use glam::Vec2;

use crate::app::App;
use crate::projectile::SHURIKEN;

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
    // `Main.player[owner]` inputs); the whips anchor their curve on the same
    // center (the `GetArmPosition` stand-in) and aim the Moon Lord arms
    // along `swing_aim` (`AI_165_Whip`'s mouse aim, `:47709`).
    let owner = app.player.as_ref().map(|p| {
        let (x, y) = p.center();
        (x, y)
    });
    let aim = if app.swing_aim.is_finite() {
        Some(Vec2::from_angle(app.swing_aim))
    } else {
        None
    };
    // The closure's disjoint field captures keep the step borrow
    // (`projectile_step`) apart from the pool/world/npcs/player set.
    app.projectile_step.tick(dt.min(0.25), |_| {
        app.projectiles.update(
            &mut app.world,
            &mut app.npcs,
            owner,
            aim,
            app.player.as_mut(),
            &mut app.item_rng,
        );
    });
    app.projectile_step.cap_backlog();
}
