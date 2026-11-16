//! The gore pool's collision seam (`tr_core::gore::GoreCollision`,
//! `Gore.cs` `Update`).
//!
//! `tr-core` cannot depend on `tr-app`'s collision ports, so the pool takes
//! the probes as a trait the App injects — the same pattern the dust pool uses
//! for its wet probe (`tr-core::particles::DustPool::set_wet_probe`,
//! `particles.rs:197-202`). Each method is a thin forward to the port:
//!
//! * `tile_collision` → [`crate::collision::tile_collision`]
//!   (`Collision.TileCollision`, `Collision.cs:2292`), with the C# `Gore.cs`
//!   call sites' defaults `fallThrough = false`, `fall2 = false`.
//! * `wet_collision` → [`crate::player::wet_collision`]
//!   (`Collision.WetCollision`, `Collision.cs:1572`).
//! * `slope_collision` → [`crate::collision::slope_collision`]
//!   (`Collision.SlopeCollision`, `Collision.cs:1791`). The C# returns a
//!   `Vector4` (X/Y the new position, Z/W the new velocity); the Rust port
//!   returns the same pair as struct fields. The port's `gravity` parameter is
//!   not carried (the C# `Gore.cs` passes `0f` at `:1037` and `1f` at `:1170`,
//!   where it only scales the slope push-down — see the note below).

use glam::Vec2;
use tr_core::gore::GoreCollision;
use tr_core::world::World;

/// The App-side implementation handed to [`tr_core::gore::GorePool::update`].
pub(crate) struct AppGoreCollision;

impl GoreCollision for AppGoreCollision {
    fn tile_collision(
        &self,
        world: &World,
        position: Vec2,
        velocity: Vec2,
        width: f32,
        height: f32,
    ) -> Vec2 {
        // `Gore.cs` calls `Collision.TileCollision(Position, Velocity, W, H)`
        // (`:944`, `:995`, `:1139`, `:1169`, `:1226`, `:1256`) — the C#
        // defaults are `fallThrough = false`, `fall2 = false`.
        let c = crate::collision::tile_collision(
            world,
            [position.x, position.y],
            [velocity.x, velocity.y],
            width,
            height,
            false,
            false,
        );
        Vec2::new(c.vel[0], c.vel[1])
    }

    fn wet_collision(&self, world: &World, position: Vec2, width: f32, height: f32) -> bool {
        // `Collision.WetCollision(this.position + this.velocity, 16, 14)`
        // (`Gore.cs:957`).
        crate::player::wet_collision(world, position.x, position.y, width, height)
    }

    fn slope_collision(
        &self,
        world: &World,
        position: Vec2,
        velocity: Vec2,
        width: f32,
        height: f32,
        _gravity: f32,
        fall: bool,
    ) -> (Vec2, Vec2) {
        // `Collision.SlopeCollision(Position, Velocity, W, H, gravity, fall)`
        // (`Gore.cs:1037`, `:1170`). The C# reads only `fall` from the extra
        // pair: its `gravity` parameter is declared at `Collision.cs:1791` and
        // never used in the body, so the port's signature without it is exact.
        let s = crate::collision::slope_collision(
            world,
            [position.x, position.y],
            [velocity.x, velocity.y],
            width,
            height,
            fall,
        );
        (Vec2::new(s.pos[0], s.pos[1]), Vec2::new(s.vel[0], s.vel[1]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The seam forwards to the ports: an empty world never blocks, so every
    /// probe returns its input unchanged.
    #[test]
    fn the_seam_forwards_an_empty_world_untouched() {
        let world = World::new("test".into(), 64, 64);
        let seam = AppGoreCollision;
        let pos = Vec2::new(100.0, 100.0);
        let vel = Vec2::new(1.5, -2.5);
        assert_eq!(seam.tile_collision(&world, pos, vel, 16.0, 14.0), vel);
        assert!(!seam.wet_collision(&world, pos, 16.0, 14.0));
        let (p, v) = seam.slope_collision(&world, pos, vel, 4.0, 4.0, 0.0, true);
        assert_eq!(p, pos);
        assert_eq!(v, vel);
    }
}
