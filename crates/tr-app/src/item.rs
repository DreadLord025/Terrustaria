//! World item drops, the first pass of `Terraria.Item`.
//!
//! Blocks broken by the player spawn a small physics entity instead of teleporting into
//! the inventory. Drops fall with gravity, bounce off solid tiles, and are magnetised to
//! the player once they are close and old enough, then added to the first matching or
//! empty hotbar slot. Rendering uses the tile's plain 16x16 frame (we do not extract the
//! separate `Item_*` sprites yet) tinted by the light map.

use glam::Vec2;
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::{TILE_PX, World};

use crate::assets;

pub const SIZE: f32 = 14.0;
/// Wall ids are stored as `WALL_ITEM_BASE + WallID` in an item slot.
pub const WALL_ITEM_BASE: u16 = 10_000;
/// Weapon/projectile items live in their own range: `WEAPON_ITEM_BASE + projectile id`.
pub const WEAPON_ITEM_BASE: u16 = 20_000;

pub fn is_wall(kind: u16) -> bool {
    (WALL_ITEM_BASE..WEAPON_ITEM_BASE).contains(&kind)
}

pub fn wall_id(kind: u16) -> u16 {
    kind - WALL_ITEM_BASE
}

pub fn is_weapon(kind: u16) -> bool {
    kind >= WEAPON_ITEM_BASE
}

pub fn weapon_id(kind: u16) -> u16 {
    kind - WEAPON_ITEM_BASE
}

/// Display name for a tile or wall item. `Terraria.Lang` would provide these.
pub fn name(kind: u16) -> &'static str {
    if is_weapon(kind) {
        return match weapon_id(kind) {
            1 => "Shuriken",
            _ => "Weapon",
        };
    }
    if is_wall(kind) {
        return match wall_id(kind) {
            1 => "Stone Wall",
            2 => "Dirt Wall",
            _ => "Wall",
        };
    }
    match kind {
        0 => "Dirt",
        1 => "Stone",
        2 => "Grass Block",
        4 => "Torch",
        5 => "Wood",
        6 => "Iron Ore",
        7 => "Copper Ore",
        8 => "Gold Ore",
        9 => "Silver Ore",
        53 => "Sand Block",
        57 => "Ash Block",
        58 => "Hellstone",
        147 => "Snow Block",
        _ => "Block",
    }
}
const MAGNET_RADIUS: f32 = 42.0;
const GRAVITY: f32 = 0.3;
const MAX_FALL: f32 = 6.0;
/// Seconds before a drop can be collected (vanilla `Item.instanced` pickup delay).
const PICKUP_DELAY: f32 = 0.35;

#[derive(Clone, Debug)]
pub struct ItemDrop {
    pub kind: u16,
    pub count: u32,
    /// Top-left of the 14x14 item box, world pixels.
    pub pos: Vec2,
    pub vel: Vec2,
    pub age: f32,
    /// Bob phase so items on the ground do not pulse in lockstep.
    pub phase: f32,
}

impl ItemDrop {
    pub fn new(kind: u16, count: u32, x: f32, y: f32, vx: f32, vy: f32, phase: f32) -> Self {
        Self {
            kind,
            count,
            pos: Vec2::new(x, y),
            vel: Vec2::new(vx, vy),
            age: 0.0,
            phase,
        }
    }

    pub fn center(&self) -> Vec2 {
        self.pos + Vec2::splat(SIZE * 0.5)
    }

    /// One fixed 60 Hz step of gravity, collision and the pickup magnet.
    /// Returns true when the drop was collected by `player`.
    pub fn update(&mut self, world: &World, player_center: Option<Vec2>, dt: f32) -> bool {
        self.age += dt;
        if let Some(pc) = player_center {
            let d = pc - self.center();
            let dist = d.length();
            if self.age > PICKUP_DELAY && dist < MAGNET_RADIUS {
                // Pull toward the player; skip the ground physics while collecting.
                let speed = if dist < 10.0 { 8.0 } else { 4.0 };
                self.vel = d.normalize_or_zero() * speed;
                self.pos += self.vel;
                return dist < 10.0;
            }
        }
        self.vel.y = (self.vel.y + GRAVITY).min(MAX_FALL);
        let nx = self.pos.x + self.vel.x;
        if crate::player::collides_rect(world, nx, self.pos.y, SIZE, SIZE) {
            self.vel.x = -self.vel.x * 0.35;
        } else {
            self.pos.x = nx;
        }
        let ny = self.pos.y + self.vel.y;
        if crate::player::collides_rect(world, self.pos.x, ny, SIZE, SIZE) {
            if self.vel.y > 0.0 {
                let ty = ((ny + SIZE) / TILE_PX).floor() as i32;
                self.pos.y = ty as f32 * TILE_PX - SIZE;
                if self.vel.y > 2.0 {
                    self.vel.y = -self.vel.y * 0.3;
                } else {
                    self.vel.y = 0.0;
                    self.vel.x *= 0.6;
                }
            } else {
                let ty = (ny / TILE_PX).floor() as i32;
                self.pos.y = (ty + 1) as f32 * TILE_PX;
                self.vel.y = 0.0;
            }
        } else {
            self.pos.y = ny;
        }
        false
    }

    /// Draws the drop in world space with a small ground bob. Returns the quad count.
    pub fn draw(
        &self,
        r: &mut tr_render::SpriteRenderer,
        lookups: &assets::Lookups,
        light: &LightMap,
        style: &LightStyle,
        anim_time: f32,
    ) -> u32 {
        let uv = if is_weapon(self.kind) {
            lookups.shuriken_uv()
        } else if is_wall(self.kind) {
            lookups.wall_icon_uv(wall_id(self.kind))
        } else {
            lookups.plain_tex(self.kind).map(|t| t.uv)
        };
        let Some(uv) = uv else {
            return 0;
        };
        let tx = (self.pos.x / TILE_PX) as i32;
        let ty = (self.pos.y / TILE_PX) as i32;
        let l = light.get(tx, ty);
        let c = style.shade(l, tx, ty, anim_time);
        let bob = (anim_time * 3.0 + self.phase).sin() * 1.5;
        r.push_sprite(
            [self.pos.x + 1.0, self.pos.y + 1.0 + bob],
            [SIZE, SIZE],
            uv,
            [c[0], c[1], c[2], 1.0],
        );
        1
    }
}

/// Hash of the item list for the G6 world-range key; includes the animation step so
/// the ground bob updates.
pub fn render_key(items: &[ItemDrop], anim_step: u32) -> u32 {
    let mut h = items.len() as u32 ^ (anim_step << 24);
    for it in items {
        h = h.wrapping_mul(31).wrapping_add(it.kind as u32);
        h = h.wrapping_mul(31).wrapping_add(it.pos.x.to_bits());
        h = h.wrapping_mul(31).wrapping_add(it.pos.y.to_bits());
        h = h.wrapping_mul(31).wrapping_add(it.count);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground() -> World {
        let mut w = World::new("items".into(), 32, 32);
        for x in 0..32 {
            for y in 20..32 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn wall_items_are_encoded_above_the_tile_range() {
        let wall = WALL_ITEM_BASE + 2;
        assert!(is_wall(wall));
        assert!(!is_wall(1));
        assert_eq!(wall_id(wall), 2);
        assert_eq!(name(wall), "Dirt Wall");
        assert_eq!(name(1), "Stone");
        let shuriken = WEAPON_ITEM_BASE + 1;
        assert!(is_weapon(shuriken));
        assert!(!is_wall(shuriken));
        assert_eq!(weapon_id(shuriken), 1);
        assert_eq!(name(shuriken), "Shuriken");
    }

    #[test]
    fn drops_fall_and_rest_on_the_ground() {
        let w = ground();
        let mut it = ItemDrop::new(0, 1, 10.0 * TILE_PX, 10.0 * TILE_PX, 0.0, 0.0, 0.0);
        for _ in 0..240 {
            assert!(!it.update(&w, None, 1.0 / 60.0));
        }
        let feet = it.pos.y + SIZE;
        assert!(
            (feet - 20.0 * TILE_PX).abs() < 2.0,
            "rested at {feet} instead of the floor"
        );
    }

    #[test]
    fn drops_bounce_off_walls() {
        let mut w = ground();
        for y in 14..20 {
            let i = w.index(14, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut it = ItemDrop::new(0, 1, 10.0 * TILE_PX, 19.0 * TILE_PX, 6.0, 0.0, 0.0);
        for _ in 0..120 {
            it.update(&w, None, 1.0 / 60.0);
        }
        assert!(it.pos.x + SIZE <= 14.0 * TILE_PX, "x {}", it.pos.x);
    }

    #[test]
    fn magnet_collects_a_drop_next_to_the_player() {
        let w = ground();
        let player = Vec2::new(16.0 * TILE_PX, 20.0 * TILE_PX - 21.0);
        let mut it = ItemDrop::new(7, 3, 16.0 * TILE_PX + 8.0, 19.0 * TILE_PX, 0.0, 0.0, 0.0);
        let mut collected = false;
        for _ in 0..240 {
            if it.update(&w, Some(player), 1.0 / 60.0) {
                collected = true;
                break;
            }
        }
        assert!(collected, "drop was never collected (pos {:?})", it.pos);
    }
}
