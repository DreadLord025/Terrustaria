//! Player physics and collision, modelled on `Player.cs`.
//!
//! Core movement: gravity, acceleration/friction, jump, 1-tile auto-step, axis-separated
//! AABB collision against solid tiles, simple water buoyancy and ground-slope support
//! (slopes 1/2). Half bricks and ceiling slopes still count as full cells.

use tr_core::tiledata as td;
use tr_core::{TILE_PX, World};

pub const WIDTH: f32 = 20.0;
pub const HEIGHT: f32 = 42.0;

const GRAVITY: f32 = 0.4;
const MAX_FALL: f32 = 10.0;
const MAX_RUN: f32 = 3.0;
const GROUND_ACCEL: f32 = 0.2;
const AIR_ACCEL: f32 = 0.1;
const GROUND_FRICTION: f32 = 0.2;
const JUMP_SPEED: f32 = 5.2;
const STEP_HEIGHT: f32 = 16.0;

#[derive(Default, Clone, Copy)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub down: bool,
}

#[derive(Clone, Debug)]
pub struct Player {
    /// Top-left of the 20x42 hitbox, world pixels.
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub on_ground: bool,
    /// 1 = right, -1 = left.
    pub facing: i32,
    /// Seconds spent in walk animation.
    pub walk_time: f32,
    pub in_water: bool,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub hurt_cd: f32,
}

impl Player {
    pub fn spawn(world: &World) -> Self {
        let sx = world.spawn_x as f32 * TILE_PX + 8.0 - WIDTH * 0.5;
        let sy = world.spawn_y as f32 * TILE_PX;
        Self {
            x: sx,
            y: sy,
            vx: 0.0,
            vy: 0.0,
            on_ground: false,
            facing: 1,
            walk_time: 0.0,
            in_water: false,
            spawn_x: sx,
            spawn_y: sy,
            hp: 100,
            max_hp: 100,
            hurt_cd: 0.0,
        }
    }

    pub fn respawn(&mut self) {
        self.x = self.spawn_x;
        self.y = self.spawn_y;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = false;
        self.walk_time = 0.0;
        self.hp = self.max_hp;
        self.hurt_cd = 0.0;
    }

    /// Contact damage / future combat. Returns true when the hit was accepted.
    pub fn hurt(&mut self, amount: i32, from_x: f32) -> bool {
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        self.hp = (self.hp - amount).max(0);
        self.hurt_cd = 0.8;
        let dir = if self.x + WIDTH * 0.5 >= from_x {
            1.0
        } else {
            -1.0
        };
        self.vx = dir * 4.0;
        self.vy = -3.0;
        self.on_ground = false;
        true
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + WIDTH * 0.5, self.y + HEIGHT * 0.5)
    }

    /// One fixed 60 Hz step.
    pub fn update(&mut self, world: &World, input: PlayerInput) {
        self.hurt_cd = (self.hurt_cd - 1.0 / 60.0).max(0.0);
        let (cx, cy) = self.center();
        self.in_water = world
            .tile((cx / TILE_PX) as i32, (cy / TILE_PX) as i32)
            .is_some_and(|t| {
                t.liquid > 0
                    && matches!(
                        t.liquid_kind(),
                        tr_core::tile::LiquidKind::Water | tr_core::tile::LiquidKind::Honey
                    )
            });

        let dir = (input.right as i32 - input.left as i32) as f32;
        let accel = if self.on_ground {
            GROUND_ACCEL
        } else {
            AIR_ACCEL
        };
        if dir != 0.0 {
            self.vx += dir * accel;
            self.facing = dir as i32;
        } else if self.on_ground {
            let f = GROUND_FRICTION.min(self.vx.abs());
            self.vx -= self.vx.signum() * f;
        }
        self.vx = self.vx.clamp(-MAX_RUN, MAX_RUN);

        let gravity = if self.in_water { 0.12 } else { GRAVITY };
        self.vy = (self.vy + gravity).min(if self.in_water { 2.0 } else { MAX_FALL });

        if input.jump {
            if self.on_ground {
                self.vy = -JUMP_SPEED;
                self.on_ground = false;
            } else if self.in_water {
                self.vy = -2.0;
            }
        }

        if self.vx != 0.0 {
            self.walk_time += 1.0 / 60.0;
        } else {
            self.walk_time = 0.0;
        }

        self.move_x(world);
        self.move_y(world, input.down);

        if self.x < 0.0 {
            self.x = 0.0;
            self.vx = 0.0;
        }
        let max_x = (world.width as f32 * TILE_PX) - WIDTH;
        if self.x > max_x {
            self.x = max_x;
            self.vx = 0.0;
        }
        if self.y > world.height as f32 * TILE_PX + 300.0 {
            self.respawn();
        }
    }

    fn move_x(&mut self, world: &World) {
        let nx = self.x + self.vx;
        if !collides_player_rect(world, nx, self.y) {
            self.x = nx;
            return;
        }
        // Auto-step one tile when walking against a ledge, like `Player.StepUp`.
        if self.on_ground || self.in_water {
            let stepped = self.y - STEP_HEIGHT;
            if !collides_player_rect(world, self.x, stepped)
                && !collides_player_rect(world, nx, stepped)
            {
                self.y = stepped;
                self.x = nx;
                return;
            }
        }
        // Resolve against the tile edge.
        if self.vx > 0.0 {
            let tx = ((self.x + WIDTH + self.vx) / TILE_PX).floor() as i32;
            self.x = tx as f32 * TILE_PX - WIDTH;
        } else if self.vx < 0.0 {
            let tx = ((self.x + self.vx) / TILE_PX).floor() as i32;
            self.x = (tx + 1) as f32 * TILE_PX;
        }
        self.vx = 0.0;
    }

    fn move_y(&mut self, world: &World, down: bool) {
        let ny = self.y + self.vy;
        if !collides_player_rect(world, self.x, ny) {
            self.y = ny;
            if !self.in_water {
                self.on_ground = false;
            }
        } else if self.vy > 0.0 {
            // Snap to the highest floor under the feet.
            let ty = ((ny + HEIGHT) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - HEIGHT;
            self.vy = 0.0;
            self.on_ground = !down;
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
            self.on_ground = false;
        }
        // Ground slopes: keep the feet on the diagonal when standing or falling onto it.
        if self.vy >= 0.0 {
            let feet = self.y + HEIGHT;
            if let Some(surface) = slope_floor(world, self.x, feet) {
                if (feet - surface).abs() <= 10.0 {
                    self.y = surface - HEIGHT;
                    self.vy = 0.0;
                    self.on_ground = true;
                }
            }
        }
    }

    /// Deterministic identity for the G6 world-range cache.
    pub fn render_key(&self) -> u32 {
        let mut h = self.x.to_bits().wrapping_mul(0x9E37_79B1);
        h ^= self.y.to_bits().wrapping_mul(0x85EB_CA77);
        h ^= (self.facing as u32) << 1;
        h ^= self.on_ground as u32;
        h ^= self.leg_frame() << 8;
        h
    }

    pub fn leg_frame(&self) -> u32 {
        if !self.on_ground {
            2
        } else if self.vx.abs() > 0.2 {
            7 + ((self.walk_time * 12.0) as u32 % 12)
        } else {
            5
        }
    }
}

/// Player movement collision: ground slopes (slope 1/2) are handled by `slope_floor`.
pub fn collides_player_rect(world: &World, x: f32, y: f32) -> bool {
    collides_filtered(world, x, y, WIDTH, HEIGHT, true)
}

pub fn collides_player(world: &World, x: f32, y: f32) -> bool {
    collides_player_rect(world, x, y)
}

/// Generic solid-tile AABB test used by NPCs and the player.
pub fn collides_rect(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    collides_filtered(world, x, y, w, h, false)
}

fn collides_filtered(world: &World, x: f32, y: f32, w: f32, h: f32, ignore_slopes: bool) -> bool {
    let x0 = (x / TILE_PX).floor() as i32;
    let x1 = ((x + w - 0.01) / TILE_PX).floor() as i32;
    let y0 = (y / TILE_PX).floor() as i32;
    let y1 = ((y + h - 0.01) / TILE_PX).floor() as i32;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            if ignore_slopes && (t.slope == 1 || t.slope == 2) {
                continue;
            }
            if (t.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(t.kind) {
                return true;
            }
        }
    }
    false
}

/// Surface height of a ground slope under the player's centre, if the feet are on it.
fn slope_floor(world: &World, x: f32, feet: f32) -> Option<f32> {
    let tx = ((x + WIDTH * 0.5) / TILE_PX).floor() as i32;
    let ty = (feet / TILE_PX).floor() as i32;
    let t = world.tile(tx, ty)?;
    if !t.active() {
        return None;
    }
    let local = ((x + WIDTH * 0.5) - tx as f32 * TILE_PX).clamp(0.0, TILE_PX);
    let top = ty as f32 * TILE_PX;
    match t.slope {
        1 => Some(top + local),
        2 => Some(top + TILE_PX - local),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    fn ground_world() -> World {
        let mut w = World::new("t".into(), 64, 128);
        for x in 0..64 {
            for y in 60..128 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 10;
        w.spawn_y = 40;
        w
    }

    #[test]
    fn falls_and_lands_on_ground() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground, "player should land, y={}", p.y);
        assert!((p.y - (60.0 * TILE_PX - HEIGHT)).abs() < 1.0, "y={}", p.y);
    }

    #[test]
    fn walks_and_is_blocked_by_walls() {
        let mut w = ground_world();
        for y in 50..60 {
            let i = w.index(20, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = Player::spawn(&w);
        for _ in 0..60 {
            p.update(&w, PlayerInput::default());
        }
        for _ in 0..240 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
        }
        assert!(p.x < 20.0 * TILE_PX, "player passed the wall: x={}", p.x);
    }

    #[test]
    fn jump_leaves_the_ground() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let grounded_y = p.y;
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        for _ in 0..10 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.y < grounded_y - 4.0, "jump did not rise: {}", p.y);
    }

    #[test]
    fn stands_on_a_ground_slope() {
        let mut w = World::new("t".into(), 32, 64);
        for x in 0..32 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = w.index(10, 29);
        w.tiles[i] = Tile {
            kind: 1,
            slope: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.spawn_x = 8;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();

        let mut p = Player::spawn(&w);
        p.x = 10.0 * TILE_PX + 8.0 - WIDTH * 0.5;
        p.y = 29.0 * TILE_PX - HEIGHT;
        for _ in 0..30 {
            p.update(&w, PlayerInput::default());
        }
        let feet = p.y + HEIGHT;
        let expected = 29.0 * TILE_PX + 8.0;
        assert!(
            (feet - expected).abs() <= 1.5,
            "feet {feet} expected {expected}"
        );
        assert!(p.on_ground);
    }
}
