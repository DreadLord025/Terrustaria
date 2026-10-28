//! Player physics and collision, modelled on `Player.cs`.
//!
//! Core movement: gravity, acceleration/friction, jump, 1-tile auto-step, axis-separated
//! AABB collision against solid tiles, simple water buoyancy and ground-slope support
//! (slopes 1/2) and half bricks/ceiling slopes during movement. Survival follows
//! `Player.cs`: a 200-tick breath meter (1 unit every 7
//! ticks underwater, 2 damage once empty), fall damage over 25 tiles, lava damage and
//! out-of-combat health regeneration. Half bricks and ceiling slopes still count as full
//! cells.

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
/// `Player.breathMax`; `breath` is in ticks.
pub const BREATH_MAX: i32 = 200;
/// `Player.breathCDMax`: ticks between breath decrements.
const BREATH_CD: i32 = 7;
/// Fall distance (tiles) before damage kicks in (`Player.cs` `num10 = 25`).
const SAFE_FALL_TILES: f32 = 25.0;
/// Seconds without damage before regeneration starts.
const REGEN_DELAY: f32 = 5.0;
/// Seconds per regenerated hit point once regeneration is active.
const REGEN_INTERVAL: f32 = 0.5;

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
    /// Remaining breath in ticks (`Player.breath`).
    pub breath: i32,
    breath_cd: i32,
    /// Seconds since the last damage; drives health regeneration.
    pub regen_time: f32,
    regen_acc: f32,
    /// Y (px) where the current fall started, for fall damage.
    fall_start: f32,
    pub in_lava: bool,
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
            breath: BREATH_MAX,
            breath_cd: 0,
            regen_time: REGEN_DELAY,
            regen_acc: 0.0,
            fall_start: sy / TILE_PX,
            in_lava: false,
        }
    }

    /// Moves the player to a pixel position without changing health or breath.
    /// Used by the `TR_XY` debug hook and future teleport effects.
    pub fn teleport(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = false;
        self.fall_start = y / TILE_PX;
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
        self.breath = BREATH_MAX;
        self.breath_cd = 0;
        self.regen_time = REGEN_DELAY;
        self.regen_acc = 0.0;
        self.fall_start = self.y / TILE_PX;
        self.in_lava = false;
    }

    /// Contact damage / future combat. Returns true when the hit was accepted.
    pub fn hurt(&mut self, amount: i32, from_x: f32) -> bool {
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        self.hp = (self.hp - amount).max(0);
        self.hurt_cd = 0.8;
        self.regen_time = 0.0;
        self.regen_acc = 0.0;
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

    /// Direct health loss with no knockback or i-frames, used by drowning and fall
    /// damage (`statLife -= ...` in `Player.cs`).
    fn lose_health(&mut self, amount: i32) {
        if self.hp <= 0 {
            return;
        }
        self.hp = (self.hp - amount).max(0);
        self.hurt_cd = self.hurt_cd.max(0.8);
        self.regen_time = 0.0;
        self.regen_acc = 0.0;
    }

    /// One fixed 60 Hz step.
    pub fn update(&mut self, world: &World, input: PlayerInput) {
        self.hurt_cd = (self.hurt_cd - 1.0 / 60.0).max(0.0);
        let (cx, cy) = self.center();
        let mid = world.tile((cx / TILE_PX) as i32, (cy / TILE_PX) as i32);
        self.in_water = mid.is_some_and(|t| {
            t.liquid > 0
                && matches!(
                    t.liquid_kind(),
                    tr_core::tile::LiquidKind::Water | tr_core::tile::LiquidKind::Honey
                )
        });
        self.in_lava =
            mid.is_some_and(|t| t.liquid > 0 && t.liquid_kind() == tr_core::tile::LiquidKind::Lava);

        // Vanilla resets the fall origin whenever the player is not falling, is wet,
        // or has just jumped.
        if self.vy <= 0.0 || self.in_water || input.jump {
            self.fall_start = self.y / TILE_PX;
        }

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
        let was_airborne = !self.on_ground;
        let fall_vy = self.vy;
        self.move_y(world, input.down);

        // `Player.cs`: landing after more than 25 tiles deals 10 damage per extra tile.
        if was_airborne && fall_vy > 0.0 && self.on_ground && !self.in_water {
            let fell = self.y / TILE_PX - self.fall_start;
            if fell > SAFE_FALL_TILES {
                let dmg = ((fell - SAFE_FALL_TILES) * 10.0) as i32;
                self.lose_health(dmg);
            }
            self.fall_start = self.y / TILE_PX;
        }

        self.update_survival();

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

    fn update_survival(&mut self) {
        if self.in_water {
            self.breath_cd += 1;
            if self.breath_cd >= BREATH_CD {
                self.breath_cd = 0;
                self.breath -= 1;
                if self.breath <= 0 {
                    self.breath = 0;
                    self.lose_health(2);
                }
            }
        } else {
            self.breath = (self.breath + 3).min(BREATH_MAX);
            self.breath_cd = 0;
        }

        if self.in_lava && self.hurt_cd <= 0.0 {
            let cx = self.x + WIDTH * 0.5;
            self.hurt(30, cx);
        }

        self.regen_time += 1.0 / 60.0;
        if self.regen_time > REGEN_DELAY && self.hp > 0 {
            self.regen_acc += 1.0 / 60.0;
            while self.regen_acc >= REGEN_INTERVAL {
                self.regen_acc -= REGEN_INTERVAL;
                self.hp = (self.hp + 1).min(self.max_hp);
            }
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
            // Snap to the highest floor under the feet. A row made only of half
            // bricks has its walkable surface 8 px below the cell top.
            let ty = ((ny + HEIGHT) / TILE_PX).floor() as i32;
            let mut half_only = true;
            let tx0 = (self.x / TILE_PX).floor() as i32;
            let tx1 = ((self.x + WIDTH - 0.01) / TILE_PX).floor() as i32;
            for tx in tx0..=tx1 {
                if let Some(t) = world.tile(tx, ty) {
                    if !t.active() || (t.kind as usize) >= td::TILE_COUNT {
                        continue;
                    }
                    if !td::TILE_SOLID.has(t.kind) || t.slope == 1 || t.slope == 2 {
                        continue;
                    }
                    if !t.half_brick() {
                        half_only = false;
                    }
                }
            }
            let top = ty as f32 * TILE_PX + if half_only { 8.0 } else { 0.0 };
            self.y = top - HEIGHT;
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
            if (t.kind as usize) >= td::TILE_COUNT || !td::TILE_SOLID.has(t.kind) {
                continue;
            }
            if ignore_slopes {
                if t.slope == 1 || t.slope == 2 {
                    continue; // ground ramps are handled by `slope_floor`
                }
                if t.half_brick() {
                    // Only the bottom 8 px are solid.
                    if y + h > ty as f32 * TILE_PX + 8.0 {
                        return true;
                    }
                    continue;
                }
                if t.slope >= 3 {
                    // Ceiling ramp: solid above the diagonal, empty below it.
                    let base_x = tx as f32 * TILE_PX;
                    let base_y = ty as f32 * TILE_PX;
                    let sx0 = x.max(base_x);
                    let sx1 = (x + w).min(base_x + TILE_PX);
                    let mut cx = sx0;
                    while cx < sx1 {
                        let local = cx - base_x;
                        let boundary = if t.slope == 3 {
                            base_y + TILE_PX - local
                        } else {
                            base_y + local
                        };
                        if y < boundary && y + h > base_y {
                            return true;
                        }
                        cx += 2.0;
                    }
                    continue;
                }
            }
            return true;
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
    fn long_falls_hurt_but_short_ones_do_not() {
        let mut w = ground_world();
        w.spawn_y = 30;
        let mut p = Player::spawn(&w);
        for _ in 0..300 {
            p.update(&w, PlayerInput::default());
        }
        // The hitbox top lands at 60 - 42/16 = 57.4 tiles, so the drop is ~27:
        // 2.4 tiles over the safe distance.
        assert!(p.on_ground);
        assert!(p.hp < 90 && p.hp > 0, "hp {}", p.hp);

        let mut w = ground_world();
        w.spawn_y = 50;
        let mut p = Player::spawn(&w);
        for _ in 0..200 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground);
        assert_eq!(p.hp, 100, "a 10-tile fall must not hurt");
    }

    #[test]
    fn drowning_drains_breath_and_damages() {
        let mut w = ground_world();
        for x in 0..64 {
            // A pool from y=40 down to the ground, so the player settles on the floor.
            for y in 40..60 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
            }
        }
        w.spawn_y = 44;
        let mut p = Player::spawn(&w);
        for _ in 0..1500 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.in_water);
        assert_eq!(p.breath, 0, "breath {}", p.breath);
        assert!(p.hp < 100, "drowning should hurt, hp {}", p.hp);
    }

    #[test]
    fn health_regenerates_after_a_delay() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.hp = 50;
        p.regen_time = 0.0;
        for _ in 0..1200 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.hp > 60, "regen only reached {} (sit still, 20 s)", p.hp);
    }

    #[test]
    fn stands_on_half_bricks_at_their_own_height() {
        let mut w = ground_world();
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i] = Tile {
                kind: 1,
                flags: FLAG_ACTIVE | tr_core::tile::FLAG_HALF_BRICK,
                ..Default::default()
            };
        }
        let mut p = Player::spawn(&w);
        for _ in 0..200 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground);
        let expected = 60.0 * TILE_PX + 8.0 - HEIGHT;
        assert!(
            (p.y - expected).abs() < 1.0,
            "y {} expected {expected}",
            p.y
        );
    }

    #[test]
    fn half_bricks_only_block_their_bottom_half() {
        let mut w = ground_world();
        let i = w.index(20, 50);
        w.tiles[i] = Tile {
            kind: 1,
            flags: FLAG_ACTIVE | tr_core::tile::FLAG_HALF_BRICK,
            ..Default::default()
        };
        assert!(collides_player_rect(
            &w,
            20.0 * TILE_PX,
            50.0 * TILE_PX + 8.0
        ));
        assert!(!collides_player_rect(
            &w,
            20.0 * TILE_PX,
            50.0 * TILE_PX + 8.0 - HEIGHT
        ));
    }

    #[test]
    fn ceiling_slopes_block_by_their_triangle() {
        let mut w = ground_world();
        let i = w.index(20, 50);
        w.tiles[i] = Tile {
            kind: 1,
            slope: 3,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        // Slope 3 is solid above the diagonal from (0, 16) to (16, 0).
        assert!(collides_player_rect(
            &w,
            20.0 * TILE_PX + 1.0,
            50.0 * TILE_PX
        ));
        assert!(!collides_player_rect(
            &w,
            20.0 * TILE_PX + 1.0,
            51.0 * TILE_PX
        ));
    }

    #[test]
    fn standing_on_a_ground_slope() {
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
