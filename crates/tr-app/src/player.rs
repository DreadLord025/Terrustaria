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

use crate::collision;

pub const WIDTH: f32 = 20.0;
pub const HEIGHT: f32 = 42.0;

const GRAVITY: f32 = 0.4;
const MAX_FALL: f32 = 10.0;
const MAX_RUN: f32 = 3.0;
const GROUND_ACCEL: f32 = 0.2;
const AIR_ACCEL: f32 = 0.1;
const GROUND_FRICTION: f32 = 0.2;
/// `Player.jumpSpeed` / `Player.jumpHeight`: holding the jump key keeps the
/// upward velocity at `JUMP_SPEED` for `JUMP_HEIGHT` ticks, so tap and hold jumps
/// reach different heights.
const JUMP_SPEED: f32 = 5.01;
const JUMP_HEIGHT: i32 = 15;
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
    /// Position at the previous fixed 60 Hz step, used to interpolate rendering
    /// between simulation ticks on high-refresh displays.
    pub prev_x: f32,
    pub prev_y: f32,
    pub vx: f32,
    pub vy: f32,
    pub on_ground: bool,
    /// 1 = right, -1 = left.
    pub facing: i32,
    /// Seconds spent in walk animation.
    pub walk_time: f32,
    pub in_water: bool,
    /// Remaining ticks of the held-jump boost (`Player.jump`).
    jump: i32,
    /// Jump was held on the previous fixed step, for press-edge detection.
    jump_held: bool,
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
    /// `Player.gfxOffY`: visual offset that hides the instant lift of `StepUp` /
    /// `StepDown`; eases back to zero so steps and slopes look smooth.
    pub gfx_off_y: f32,
    prev_gfx_off_y: f32,
    /// `Player.stepSpeed`: how fast `gfxOffY` eases out.
    step_speed: f32,
    /// `Player.stairFall`: slopes may be fallen through this tick.
    stair_fall: bool,
}

impl Player {
    pub fn spawn(world: &World) -> Self {
        let sx = world.spawn_x as f32 * TILE_PX + 8.0 - WIDTH * 0.5;
        let sy = world.spawn_y as f32 * TILE_PX;
        Self {
            x: sx,
            y: sy,
            prev_x: sx,
            prev_y: sy,
            vx: 0.0,
            vy: 0.0,
            on_ground: false,
            facing: 1,
            walk_time: 0.0,
            in_water: false,
            jump: 0,
            jump_held: false,
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
            gfx_off_y: 0.0,
            prev_gfx_off_y: 0.0,
            step_speed: 1.0,
            stair_fall: false,
        }
    }

    /// Moves the player to a pixel position without changing health or breath.
    /// Used by the `TR_XY` debug hook and future teleport effects.
    pub fn teleport(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
        self.prev_x = x;
        self.prev_y = y;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = false;
        self.jump = 0;
        self.fall_start = y / TILE_PX;
        self.gfx_off_y = 0.0;
        self.prev_gfx_off_y = 0.0;
    }

    pub fn respawn(&mut self) {
        self.x = self.spawn_x;
        self.y = self.spawn_y;
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.vx = 0.0;
        self.vy = 0.0;
        self.on_ground = false;
        self.jump = 0;
        self.jump_held = false;
        self.walk_time = 0.0;
        self.hp = self.max_hp;
        self.hurt_cd = 0.0;
        self.breath = BREATH_MAX;
        self.breath_cd = 0;
        self.regen_time = REGEN_DELAY;
        self.regen_acc = 0.0;
        self.fall_start = self.y / TILE_PX;
        self.in_lava = false;
        self.gfx_off_y = 0.0;
        self.prev_gfx_off_y = 0.0;
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
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.prev_gfx_off_y = self.gfx_off_y;
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

        let jump_pressed = input.jump && !self.jump_held;
        self.jump_held = input.jump;
        if input.jump {
            if self.on_ground && jump_pressed {
                self.vy = -JUMP_SPEED;
                self.jump = JUMP_HEIGHT;
                self.on_ground = false;
            } else if self.jump > 0 {
                if self.vy == 0.0 {
                    self.jump = 0;
                } else {
                    self.vy = -JUMP_SPEED;
                    self.jump -= 1;
                }
            } else if self.in_water {
                self.vy = -2.0;
            }
        } else {
            self.jump = 0;
        }

        if self.vx != 0.0 {
            self.walk_time += 1.0 / 60.0;
        } else {
            self.walk_time = 0.0;
        }

        let was_airborne = !self.on_ground;
        let fall_vy = self.vy;
        self.move_vanilla(world, input.down, gravity);

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

    /// The original's movement pipeline (`Player.cs`): `TileCollision` resolves the
    /// AABB against solid tiles and slope triangles, `StepDown`/`StepUp` lift the
    /// hitbox over one-tile ledges while `gfxOffY` hides the pop, and
    /// `SlopeCollision` slides the feet along ramps.
    fn move_vanilla(&mut self, world: &World, down: bool, gravity: f32) {
        // `Player.cs`: the visual offset eases back to zero at walk speed.
        let decay = (1.0 + self.vx.abs() / 3.0) * self.step_speed;
        if self.gfx_off_y > 0.0 {
            self.gfx_off_y = (self.gfx_off_y - decay).max(0.0);
        } else if self.gfx_off_y < 0.0 {
            self.gfx_off_y = (self.gfx_off_y + decay).min(0.0);
        }
        self.gfx_off_y = self.gfx_off_y.clamp(-32.0, 32.0);

        let mut pos = [self.x, self.y];
        let mut vel = [self.vx, self.vy];
        // `Player.cs` runs the step helpers on the pre-collision velocity (which
        // still carries this tick's gravity), then resolves the AABB.
        if vel[1] == gravity {
            collision::step_down(
                world,
                &mut pos,
                &mut vel,
                WIDTH,
                HEIGHT,
                &mut self.step_speed,
                &mut self.gfx_off_y,
            );
        }
        if vel[1] >= gravity && !down {
            collision::step_up(
                world,
                &mut pos,
                &mut vel,
                WIDTH,
                HEIGHT,
                &mut self.step_speed,
                &mut self.gfx_off_y,
                false,
            );
        }
        let tc = collision::tile_collision(world, pos, vel, WIDTH, HEIGHT, false, false);
        if tc.up {
            self.jump = 0;
        }
        vel = tc.vel;
        pos[0] += vel[0];
        pos[1] += vel[1];
        if down {
            self.stair_fall = true;
        }
        let sc = collision::slope_collision(world, pos, vel, WIDTH, HEIGHT, self.stair_fall);
        self.stair_fall = sc.stair_fall;
        if sc.stair && (sc.pos[1] - pos[1]).abs() > 8.0 + vel[0].abs() {
            self.gfx_off_y -= sc.pos[1] - pos[1];
            self.step_speed = 4.0;
        }
        pos = sc.pos;
        vel = sc.vel;
        self.x = pos[0];
        self.y = pos[1];
        self.vx = vel[0];
        self.vy = vel[1];
        // The original gates jumps on `velocity.Y == 0f` after collision.
        self.on_ground = self.vy == 0.0;
    }

    /// Position between the previous and current fixed steps, for smooth rendering.
    /// The visual step offset is interpolated too and baked into `y`.
    pub fn interpolated(&self, alpha: f32) -> Self {
        let a = alpha.clamp(0.0, 1.0);
        let mut out = self.clone();
        out.x = self.prev_x + (self.x - self.prev_x) * a;
        out.y = self.prev_y + (self.y - self.prev_y) * a;
        out.y += self.prev_gfx_off_y + (self.gfx_off_y - self.prev_gfx_off_y) * a;
        out.prev_x = out.x;
        out.prev_y = out.y;
        out.gfx_off_y = 0.0;
        out.prev_gfx_off_y = 0.0;
        out
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
                if t.slope() == 1 || t.slope() == 2 {
                    continue; // ground ramps are handled by `slope_floor`
                }
                if t.half_brick() {
                    // Only the bottom 8 px are solid.
                    if y + h > ty as f32 * TILE_PX + 8.0 {
                        return true;
                    }
                    continue;
                }
                if t.slope() >= 3 {
                    // Ceiling ramp: solid above the diagonal, empty below it.
                    let base_x = tx as f32 * TILE_PX;
                    let base_y = ty as f32 * TILE_PX;
                    let sx0 = x.max(base_x);
                    let sx1 = (x + w).min(base_x + TILE_PX);
                    let mut cx = sx0;
                    while cx < sx1 {
                        let local = cx - base_x;
                        let boundary = if t.slope() == 3 {
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
    fn walks_down_a_slope_smoothly() {
        let mut w = World::new("t".into(), 64, 128);
        for x in 0..64 {
            let top = if x < 20 { 60 } else { 61 };
            for y in top..100 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = w.index(19, 60);
        w.tiles[i].set_slope(1);
        w.spawn_x = 10;
        w.spawn_y = 58;
        let mut p = Player::spawn(&w);
        for _ in 0..120 { p.update(&w, PlayerInput::default()); }
        let mut last = p.y + p.gfx_off_y;
        let mut max_step: f32 = 0.0;
        for _ in 0..180 {
            p.update(&w, PlayerInput { right: true, ..Default::default() });
            let vis = p.y + p.gfx_off_y;
            max_step = max_step.max((vis - last).abs());
            last = vis;
        }
        assert!(p.y > 56.0 * TILE_PX, "should have descended, y={}", p.y);
        assert!(max_step < 8.0, "slope caused a {max_step} px visual jump");
    }

    #[test]
    fn walks_up_a_one_tile_step_smoothly() {
        let mut w = ground_world();
        // A one-tile step at x=20: `Collision.StepUp` lifts the hitbox while
        // `gfxOffY` keeps the sprite at the old height, so the climb is smooth.
        for x in 20..30 {
            let i = w.index(x, 59);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut last = p.y + p.gfx_off_y;
        let mut max_step: f32 = 0.0;
        let mut stood_on_step = false;
        for k in 0..240 {
            p.update(&w, PlayerInput { right: true, ..Default::default() });
            let vis = p.y + p.gfx_off_y;
            max_step = max_step.max((vis - last).abs());
            last = vis;
            if p.x + WIDTH > 20.0 * TILE_PX && p.x < 30.0 * TILE_PX {
                stood_on_step |= (p.y + HEIGHT - 59.0 * TILE_PX).abs() < 1.0;
            }
        }
        assert!(
            p.x + WIDTH > 21.0 * TILE_PX,
            "player never climbed the step: x={}",
            p.x
        );
        assert!(stood_on_step, "feet never rested on the step top");
        // The original eases `gfxOffY` at `(1 + |vx| / 3) * stepSpeed` px per tick
        // (5 px/tick at walk speed with stepSpeed 2.5); what must never happen is
        // the instant 16 px teleport of a naive snap.
        assert!(max_step < 8.0, "step caused a {max_step} px visual jump");
    }

    #[test]
    fn holding_jump_rises_higher_than_tapping() {
        let w = ground_world();
        let mut tap = Player::spawn(&w);
        for _ in 0..120 {
            tap.update(&w, PlayerInput::default());
        }
        let mut tap_peak = tap.y;
        tap.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        for _ in 0..60 {
            tap.update(&w, PlayerInput::default());
            tap_peak = tap_peak.min(tap.y);
        }

        let mut hold = Player::spawn(&w);
        for _ in 0..120 {
            hold.update(&w, PlayerInput::default());
        }
        let mut hold_peak = hold.y;
        for _ in 0..30 {
            hold.update(
                &w,
                PlayerInput {
                    jump: true,
                    ..Default::default()
                },
            );
            hold_peak = hold_peak.min(hold.y);
        }
        for _ in 0..60 {
            hold.update(&w, PlayerInput::default());
            hold_peak = hold_peak.min(hold.y);
        }
        assert!(
            hold_peak + 20.0 < tap_peak,
            "hold peak {hold_peak} vs tap peak {tap_peak}"
        );
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
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        w.tiles[i].set_slope(3);
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
            ..Default::default()
        };
        w.tiles[i].set_slope(1);
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.spawn_x = 8;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();

        let mut p = Player::spawn(&w);
        // Left edge six pixels into the ramp: `SlopeCollision` rests slope 1 on
        // the diagonal at the hitbox's left edge.
        p.x = 10.0 * TILE_PX + 6.0;
        p.y = 29.0 * TILE_PX - HEIGHT;
        for _ in 0..30 {
            p.update(&w, PlayerInput::default());
        }
        let feet = p.y + HEIGHT;
        let expected = 29.0 * TILE_PX + (p.x - 10.0 * TILE_PX);
        assert!(
            (feet - expected).abs() <= 1.5,
            "feet {feet} expected {expected}"
        );
        assert!(p.on_ground);
    }
}
