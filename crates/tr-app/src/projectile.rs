//! First projectile pass: `Projectile.cs` type 3, the shuriken.
//!
//! Projectiles are simulated at a fixed 60 Hz, move in two sub-steps so a 11 px/tick
//! shuriken cannot tunnel through a tile, bounce off solid tiles with 0.6 damping, damage
//! the first NPC they touch and expire after a couple of seconds. They are drawn in the
//! always-rebuilt UI range (screen space) so the cached world range is unaffected.

use glam::Vec2;
use tr_core::lighting::LightMap;
use tr_core::{TILE_PX, World};
use tr_render::sprite::SpriteRenderer;

use crate::assets;
use crate::npc::Npc;

pub const SHURIKEN: u16 = 1;
const SIZE: f32 = 6.0;

#[derive(Clone, Debug)]
pub struct Projectile {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub age: f32,
    pub damage: i32,
    bounces: u8,
    pub alive: bool,
}

impl Projectile {
    pub fn shuriken(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::new(vx, vy),
            rot: 0.0,
            age: 0.0,
            damage: 10,
            bounces: 0,
            alive: true,
        }
    }

    /// One fixed 60 Hz step. `npcs` are hit front-to-back, like `Projectile.Damage`.
    pub fn update(&mut self, world: &World, npcs: &mut [Npc], dt: f32) {
        if !self.alive {
            return;
        }
        self.age += dt;
        self.rot += if self.vel.x >= 0.0 { 0.42 } else { -0.42 };
        for _ in 0..2 {
            let step = self.vel * 0.5;
            let nx = self.pos.x + step.x;
            if crate::player::collides_rect(world, nx, self.pos.y, SIZE, SIZE) {
                self.vel.x = -self.vel.x * 0.6;
                self.bounces += 1;
            } else {
                self.pos.x = nx;
            }
            let ny = self.pos.y + step.y;
            if crate::player::collides_rect(world, self.pos.x, ny, SIZE, SIZE) {
                self.vel.y = -self.vel.y * 0.6;
                self.bounces += 1;
            } else {
                self.pos.y = ny;
            }
        }
        let center = self.pos + Vec2::splat(SIZE * 0.5);
        for n in npcs.iter_mut() {
            if n.hp > 0 && n.contains(center.x, center.y) {
                n.hurt(self.damage, center.x);
                self.alive = false;
                break;
            }
        }
        if self.age > 2.0 || self.bounces > 6 {
            self.alive = false;
        }
    }

    pub fn draw(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        camera: Vec2,
        zoom: f32,
        screen: Vec2,
    ) {
        let Some(uv) = lookups.shuriken_uv() else {
            return;
        };
        let tx = (self.pos.x / TILE_PX) as i32;
        let ty = (self.pos.y / TILE_PX) as i32;
        let c = if light.w > 0 && light.h > 0 {
            light.get(
                tx.clamp(light.x0, light.x0 + light.w - 1),
                ty.clamp(light.y0, light.y0 + light.h - 1),
            )
        } else {
            [1.0; 3]
        };
        let pos = [
            (self.pos.x - camera.x) * zoom + screen.x * 0.5,
            (self.pos.y - camera.y) * zoom + screen.y * 0.5,
        ];
        // The 22x22 sheet is drawn around the 6x6 collision centre; rotate about it.
        let size = 16.0 * zoom;
        let half = size * 0.5;
        let (s, co) = self.rot.sin_cos();
        let top_left = [
            pos[0] + SIZE * 0.5 * zoom - (half * co - half * s),
            pos[1] + SIZE * 0.5 * zoom - (half * s + half * co),
        ];
        r.push_ui_sprite_rot(
            top_left,
            [size, size],
            uv,
            [c[0], c[1], c[2], 1.0],
            self.rot,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground() -> World {
        let mut w = World::new("proj".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 10;
        w.spawn_y = 25;
        w.compute_column_surface();
        w
    }

    #[test]
    fn shuriken_hits_a_slime() {
        let w = ground();
        let mut slime = Npc::spawn_slime(&w, 14).unwrap();
        let hp = slime.hp;
        let mut p = Projectile::shuriken(10.0 * TILE_PX, 29.0 * TILE_PX, 10.0, 0.0);
        for _ in 0..120 {
            p.update(&w, std::slice::from_mut(&mut slime), 1.0 / 60.0);
            if !p.alive {
                break;
            }
        }
        assert!(!p.alive, "shuriken should hit or expire");
        assert!(slime.hp < hp, "slime hp {} stayed {hp}", slime.hp);
    }

    #[test]
    fn shuriken_bounces_off_a_wall_then_expires() {
        let mut w = ground();
        for y in 20..30 {
            let i = w.index(20, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = Projectile::shuriken(16.0 * TILE_PX, 28.0 * TILE_PX, 10.0, 0.0);
        for _ in 0..240 {
            p.update(&w, &mut [], 1.0 / 60.0);
        }
        assert!(!p.alive);
        assert!(p.bounces > 0, "never touched the wall");
    }

    #[test]
    fn shuriken_ignores_npcs_out_of_reach() {
        let w = ground();
        let mut slime = Npc::spawn_slime(&w, 20).unwrap();
        let hp = slime.hp;
        let mut p = Projectile::shuriken(10.0 * TILE_PX, 28.0 * TILE_PX, 0.0, -6.0);
        for _ in 0..90 {
            p.update(&w, std::slice::from_mut(&mut slime), 1.0 / 60.0);
        }
        assert_eq!(slime.hp, hp);
        assert_eq!(slime.kind, NpcKind::Slime);
    }
}
