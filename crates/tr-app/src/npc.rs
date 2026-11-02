//! First enemy pass: a hopping blue slime, a flying demon eye and a walking zombie,
//! modelled on the simplest `NPC.cs` behaviours.

use tr_core::{World, TILE_PX};

use crate::player::{collides_rect, Player};

const GRAVITY: f32 = 0.4;
const MAX_FALL: f32 = 8.0;

/// `Collision.WetCollision` (simplified): any water tile overlapping the body.
fn is_wet(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    let x0 = (x / TILE_PX).floor() as i32;
    let x1 = ((x + w - 0.01) / TILE_PX).floor() as i32;
    let y0 = (y / TILE_PX).floor() as i32;
    let y1 = ((y + h - 0.01) / TILE_PX).floor() as i32;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if world.tile(tx, ty).is_some_and(|t| t.liquid > 0) {
                return true;
            }
        }
    }
    false
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NpcKind {
    Slime,
    DemonEye,
    Zombie,
    KingSlime,
}

impl NpcKind {
    pub fn to_u16(self) -> u16 {
        match self {
            NpcKind::Slime => 0,
            NpcKind::DemonEye => 1,
            NpcKind::Zombie => 2,
            NpcKind::KingSlime => 3,
        }
    }

    pub fn from_u16(v: u16) -> Option<Self> {
        Some(match v {
            0 => NpcKind::Slime,
            1 => NpcKind::DemonEye,
            2 => NpcKind::Zombie,
            3 => NpcKind::KingSlime,
            _ => return None,
        })
    }

    pub fn size(self) -> (f32, f32) {
        match self {
            NpcKind::Slime => (30.0, 24.0),
            NpcKind::DemonEye => (30.0, 26.0),
            NpcKind::Zombie => (18.0, 40.0),
            NpcKind::KingSlime => (98.0, 92.0),
        }
    }

    fn max_hp(self) -> i32 {
        match self {
            NpcKind::Slime => 30,
            NpcKind::DemonEye => 25,
            NpcKind::Zombie => 45,
            NpcKind::KingSlime => 2000,
        }
    }

    fn contact_damage(self) -> i32 {
        match self {
            NpcKind::Slime => 12,
            NpcKind::DemonEye => 14,
            NpcKind::Zombie => 14,
            NpcKind::KingSlime => 40,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Npc {
    pub kind: NpcKind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub on_ground: bool,
    pub facing: i32,
    pub hop_timer: f32,
    pub anim: f32,
    pub hurt_cd: f32,
    /// `NPC.frame`: the rendered frame row, advanced by [`Npc::find_frame`] in
    /// the fixed step (the original advances animation in the sim too —
    /// `FindFrame` runs from `NPC.Update`, `NPC.cs:66684`).
    pub frame: u32,
    /// `NPC.frameCounter`: the per-kind accumulator `FindFrame` checks.
    frame_counter: f32,
    /// `NPC.rotation`: the aim tilt. Only the demon eye sets it
    /// (`NPC.cs:74019-74031`: `atan2(velocity.Y, velocity.X)`, `+ π` when
    /// `velocity.X < 0`).
    pub rotation: f32,
    /// Accumulated daylight burn time (zombies only).
    burn: f32,
    /// A boss that just landed wants the app to spawn a minion (`AI_015`).
    pub wants_minion: bool,
    /// `ai[3]`: x position while descending through water, for the bounce flip.
    wet_x: f32,
}

impl Npc {
    pub fn spawn_slime(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::Slime.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, top).is_some_and(|t| t.liquid > 128) {
            return None;
        }
        let mut seed = (x_tile as u32).wrapping_mul(2_654_435_761);
        seed ^= seed >> 15;
        Some(Self::new(
            NpcKind::Slime,
            x,
            y,
            if seed & 1 == 0 { 1 } else { -1 },
            0.4 + (seed % 80) as f32 * 0.01,
        ))
    }

    /// `NPCID.KingSlime`: the first boss, spawned by the `B` debug key or `TR_BOSS`.
    pub fn spawn_king_slime(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::KingSlime.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        let mut npc = Self::new(NpcKind::KingSlime, x, y, -1, 1.2);
        npc.hp = npc.max_hp;
        Some(npc)
    }

    /// Zombies walk on the surface and burn away at dawn.
    pub fn spawn_zombie(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::Zombie.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, top).is_some_and(|t| t.liquid > 128) {
            return None;
        }
        Some(Self::new(NpcKind::Zombie, x, y, -1, 0.0))
    }

    /// Demon eyes spawn a few tiles above the surface and ignore gravity.
    pub fn spawn_eye(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::DemonEye.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h - 4.0 * TILE_PX;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        Some(Self::new(NpcKind::DemonEye, x, y, 1, 0.0))
    }

    fn new(kind: NpcKind, x: f32, y: f32, facing: i32, hop_timer: f32) -> Self {
        Self {
            kind,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            hp: kind.max_hp(),
            max_hp: kind.max_hp(),
            on_ground: false,
            facing,
            hop_timer,
            anim: 0.0,
            hurt_cd: 0.0,
            frame: 0,
            frame_counter: 0.0,
            rotation: 0.0,
            burn: 0.0,
            wants_minion: false,
            wet_x: 0.0,
        }
    }

    pub fn center(&self) -> (f32, f32) {
        let (w, h) = self.kind.size();
        (self.x + w * 0.5, self.y + h * 0.5)
    }

    pub fn contains(&self, wx: f32, wy: f32) -> bool {
        let (w, h) = self.kind.size();
        wx >= self.x - 4.0 && wx <= self.x + w + 4.0 && wy >= self.y - 4.0 && wy <= self.y + h + 4.0
    }

    pub fn hurt(&mut self, amount: i32, from_x: f32) -> bool {
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        self.hp -= amount;
        self.hurt_cd = 0.18;
        let (w, _) = self.kind.size();
        let dir = if self.x + w * 0.5 >= from_x {
            1.0
        } else {
            -1.0
        };
        self.vx = dir * 1.6;
        self.vy = -2.0;
        self.on_ground = false;
        true
    }

    pub fn update(&mut self, world: &World, player: &mut Player, dt: f32) {
        self.hurt_cd = (self.hurt_cd - dt).max(0.0);
        self.anim += dt;
        match self.kind {
            NpcKind::Slime => self.update_slime(world, player, dt),
            NpcKind::DemonEye => self.update_eye(world, player, dt),
            NpcKind::Zombie => self.update_zombie(world, player, dt),
            NpcKind::KingSlime => self.update_king_slime(world, player, dt),
        }
        self.find_frame();
        self.contact(player);
    }

    /// `NPC.FindFrame` (`NPC.cs:66684`) for the four ported kinds. `num` (the
    /// frame height unit) cancels out because frames are tracked as row indices;
    /// the per-kind frame counts are `Main.npcFrameCount` (`Main.cs:67082`):
    /// slime 2, demon eye 2, zombie 3, King Slime 6. `num2` is the motion class
    /// (`NPC.cs:66698-66703`): 2 jumping up, 3 falling, 1 moving, 0 idle —
    /// `aiAction == 4` (attacking) never happens for these kinds in the port.
    fn find_frame(&mut self) {
        let num2 = if self.vy < 0.0 {
            2
        } else if self.vy > 0.0 {
            3
        } else if self.vx != 0.0 {
            1
        } else {
            0
        };
        match self.kind {
            // Case 1 — blue slime (`NPC.cs:70801-70826`): the counter ticks once
            // per step, again when moving and a third time while attacking; at
            // 8.0 the frame advances and wraps over the 2-frame strip.
            NpcKind::Slime => {
                self.frame_counter += 1.0 + (num2 > 0) as i32 as f32 + (num2 == 4) as i32 as f32;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 2 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            // Case 50 — King Slime (`NPC.cs:73380-73414`): airborne snaps to
            // frame 4, holds it 4 steps, then frame 5; landing returns to 4 and
            // the grounded cycle runs frames 0..=3 (never reaching 4).
            NpcKind::KingSlime => {
                if self.vy != 0.0 {
                    if self.frame < 4 {
                        self.frame = 4;
                        self.frame_counter = 0.0;
                    }
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 4.0 {
                        self.frame = 5;
                    }
                } else {
                    if self.frame >= 5 {
                        self.frame = 4;
                        self.frame_counter = 0.0;
                    }
                    self.frame_counter +=
                        1.0 + (num2 > 0) as i32 as f32 + (num2 == 4) as i32 as f32;
                    if self.frame_counter >= 8.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                        if self.frame >= 4 {
                            self.frame = 0;
                        }
                    }
                }
            }
            // Case 2 — demon eye (`NPC.cs:73994-74057`): a plain 8-step 2-frame
            // cycle, plus the aim: type 2 is in the first rotation branch
            // (`NPC.cs:74019-74031`) — `rotation = atan2(vy, vx)`, `+ π` when
            // moving left, and the sprite mirrors (`spriteDirection`) so its
            // pupil follows the flight direction. `rotation` freezes while
            // `velocity.X == 0` exactly like the original (neither `if` fires).
            NpcKind::DemonEye => {
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 2 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
                if self.vx > 0.0 {
                    self.rotation = self.vy.atan2(self.vx);
                } else if self.vx < 0.0 {
                    // The original adds the literal `3.14f`, not PI.
                    self.rotation = self.vy.atan2(self.vx) + 3.14;
                }
            }
            // Case 3 — zombie (`NPC.cs:76324-76389`): falling or moving against
            // the facing direction holds the lurch frame 2; standing still is
            // frame 0; walking accumulates `|velocity.X|` through the band
            // 0 → 1 → 2 → 1 at thresholds 8/16/24/32 and restarts.
            NpcKind::Zombie => {
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        // The original leaves `frame.Y` on its previous value
                        // here — the restart tick keeps frame 1.
                        self.frame_counter = 0.0;
                    }
                }
            }
        }
    }

    fn update_slime(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (w, h) = NpcKind::Slime.size();
        let (pcx, _) = player.center();
        let (cx, _) = self.center();
        self.hop_timer -= 1.0 / 60.0;
        let near = (pcx - cx).abs() < 60.0 * TILE_PX;
        if self.on_ground && self.hop_timer <= 0.0 && near {
            self.facing = if pcx >= cx { 1 } else { -1 };
            self.vx = self.facing as f32 * 1.4;
            self.vy = -4.2;
            self.on_ground = false;
            self.hop_timer = 1.0 + (self.anim * 7.0).sin().abs() * 0.5;
        }
        // `NPC.UpdateNPC_Gravity`: water lowers gravity and the fall cap.
        let wet = is_wet(world, self.x, self.y, w, h);
        if wet {
            self.vy = (self.vy + 0.2).min(7.0);
        } else {
            self.vy = (self.vy + GRAVITY).min(MAX_FALL);
        }

        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            self.vx = -self.vx * 0.4;
        }
        let ny = self.y + self.vy;
        let mut collide_y = false;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
            self.vx *= 0.6;
            collide_y = true;
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
            collide_y = true;
        }
        // `AI_001_Slimes` water tail: pushed up every tick, with a small bounce when
        // it touches the floor or the surface, so slimes bob at the water line.
        if wet {
            if collide_y {
                self.vy = -2.0;
            }
            if self.vy < 0.0 && self.wet_x == self.x {
                self.facing = -self.facing;
                self.hop_timer = 200.0 / 60.0;
            }
            if self.vy > 0.0 {
                self.wet_x = self.x;
            }
            if self.vy > 2.0 {
                self.vy *= 0.9;
            }
            self.vy -= 0.5;
            if self.vy < -4.0 {
                self.vy = -4.0;
            }
        }
        if self.vx != 0.0 {
            self.facing = if self.vx > 0.0 { 1 } else { -1 };
        }
        if self.y > world.height as f32 * TILE_PX + 200.0 {
            self.hp = 0;
        }
    }

    fn update_eye(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (w, h) = NpcKind::DemonEye.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let d = (dx * dx + dy * dy).sqrt().max(1.0);
        self.vx = (self.vx + dx / d * 0.12).clamp(-3.0, 3.0);
        self.vy = (self.vy + dy / d * 0.12).clamp(-3.0, 3.0);
        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            self.vx = -self.vx * 0.5;
        }
        let bob = (self.anim * 3.0).sin() * 0.4;
        let ny = self.y + self.vy + bob;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
        } else {
            self.vy = -self.vy * 0.5;
        }
        self.facing = if dx >= 0.0 { 1 } else { -1 };
        if self.y > world.height as f32 * TILE_PX + 200.0 {
            self.hp = 0;
        }
    }

    /// `AI_003_Fighters`: walk toward the player and jump when the path is blocked.
    fn update_zombie(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (w, h) = NpcKind::Zombie.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let near = dx.abs() < 80.0 * TILE_PX && dy.abs() < 40.0 * TILE_PX;
        if near {
            self.facing = if dx >= 0.0 { 1 } else { -1 };
            self.vx = self.facing as f32 * 0.6;
        } else {
            self.vx *= 0.9;
        }
        self.vy = (self.vy + GRAVITY).min(MAX_FALL);

        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else if self.on_ground {
            // Jump a one-tile ledge or wall instead of grinding into it.
            self.vy = -6.2;
            self.on_ground = false;
        } else {
            self.vx = 0.0;
        }

        let ny = self.y + self.vy;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
        }

        if world.day_time {
            self.burn += 1.0 / 60.0;
            if self.burn >= 1.0 {
                self.burn -= 1.0;
                self.hp -= 6;
            }
        } else {
            self.burn = 0.0;
        }
        if self.y > world.height as f32 * TILE_PX + 200.0 {
            self.hp = 0;
        }
    }

    /// Simplified `AI_015_KingSlime`: big hops toward the player, a teleport when the
    /// player escapes, and a burst of blue slimes when it lands. Minion spawning is
    /// reported through `spawn_minion` so the app can add them to its NPC list.
    fn update_king_slime(&mut self, world: &World, player: &mut Player, dt: f32) {
        let (w, h) = NpcKind::KingSlime.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let dx = pcx - cx;
        self.facing = if dx >= 0.0 { 1 } else { -1 };
        self.hop_timer -= dt;
        if self.on_ground && self.hop_timer <= 0.0 {
            self.vx = self.facing as f32 * 2.4;
            self.vy = -8.0;
            self.on_ground = false;
            self.hop_timer = 1.4 + (self.anim * 2.7).sin().abs() * 0.8;
        }
        // Teleport next to the player when they outrun the boss.
        if (dx.abs() > 70.0 * TILE_PX || (pcy - cy).abs() > 50.0 * TILE_PX) && self.on_ground {
            let side = if dx >= 0.0 { -1 } else { 1 };
            let tx = ((pcx / TILE_PX) as i32 + side * 10).clamp(4, world.width - 5);
            let top = world.surface_of(tx);
            if top < world.height - 2 {
                let (w2, h2) = NpcKind::KingSlime.size();
                let nx = tx as f32 * TILE_PX + 8.0 - w2 * 0.5;
                let ny = top as f32 * TILE_PX - h2;
                if !collides_rect(world, nx, ny, w2, h2) {
                    self.x = nx;
                    self.y = ny;
                    self.vx = 0.0;
                    self.vy = 0.0;
                    self.on_ground = true;
                }
            }
        }
        self.vy = (self.vy + GRAVITY).min(MAX_FALL);
        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            self.vx = -self.vx * 0.4;
        }
        let ny = self.y + self.vy;
        let was_air = !self.on_ground;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
            self.vx *= 0.5;
            if was_air {
                self.wants_minion = true;
            }
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
        }
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    fn contact(&self, player: &mut Player) {
        let (w, h) = self.kind.size();
        if self.x < player.x + crate::player::WIDTH
            && self.x + w > player.x
            && self.y < player.y + crate::player::HEIGHT
            && self.y + h > player.y
        {
            let (cx, _) = self.center();
            player.hurt(self.kind.contact_damage(), cx);
        }
    }

    pub fn render_key(&self) -> u32 {
        let mut h = self.x.to_bits().wrapping_mul(0x9E37_79B1);
        h ^= self.y.to_bits().wrapping_mul(0x85EB_CA77);
        h ^= (self.hp.max(0) as u32) << 4;
        h ^= (self.facing as u32) << 1;
        h ^= (self.hurt_cd > 0.0) as u32;
        // The rendered frame and the eye's aim tilt are the animation state the
        // world pass bakes in (`frame_counter`/`anim` only matter through them).
        h ^= self.frame << 2;
        h ^= self.rotation.to_bits().wrapping_mul(0x27D4_EB2F);
        h ^= match self.kind {
            NpcKind::Slime => 0,
            NpcKind::DemonEye => 1,
            NpcKind::Zombie => 2,
            NpcKind::KingSlime => 3,
        } << 5;
        h
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }
}

/// Hash of all NPCs for the G6 world-range key.
pub fn render_key(npcs: &[Npc]) -> u32 {
    let mut h = npcs.len() as u32;
    for n in npcs {
        h = h.wrapping_mul(31).wrapping_add(n.render_key());
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    fn ground() -> World {
        let mut w = World::new("t".into(), 64, 64);
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
        w.spawn_x = 20;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn slime_falls_onto_the_ground_and_hurts_the_player() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut slime = Npc::spawn_slime(&w, 20).unwrap();
        for _ in 0..120 {
            slime.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(slime.on_ground);
        assert!(player.hp < player.max_hp, "hp {}", player.hp);
    }

    #[test]
    fn demon_eye_reaches_the_player() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut eye = Npc::spawn_eye(&w, 20).unwrap();
        assert_eq!(eye.kind, NpcKind::DemonEye);
        for _ in 0..240 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(player.hp < player.max_hp, "hp {}", player.hp);
    }

    #[test]
    fn zombie_walks_to_the_player_and_burns_at_dawn() {
        let mut w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut zombie = Npc::spawn_zombie(&w, 30).unwrap();
        assert_eq!(zombie.kind, NpcKind::Zombie);
        let start_x = zombie.x;
        for _ in 0..600 {
            zombie.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(zombie.x < start_x, "zombie walked the wrong way");
        assert!(player.hp < player.max_hp, "hp {}", player.hp);

        // Daylight burns the zombie away.
        w.day_time = true;
        let hp = zombie.hp;
        for _ in 0..600 {
            zombie.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(zombie.hp < hp, "hp {} stayed at {hp}", zombie.hp);
    }

    #[test]
    fn king_slime_hops_toward_the_player_and_teleports() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut king = Npc::spawn_king_slime(&w, 34).unwrap();
        assert_eq!(king.hp, 2000);
        let start_x = king.x;
        for _ in 0..1200 {
            king.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(king.x < start_x, "boss did not approach");
        assert!(player.hp < player.max_hp, "hp {}", player.hp);

        // Outrun the boss: it should teleport back next to the player.
        player.x = 4.0 * TILE_PX;
        player.y = 30.0 * TILE_PX - crate::player::HEIGHT;
        for _ in 0..120 {
            king.update(&w, &mut player, 1.0 / 60.0);
        }
        let dx = ((king.x + 49.0) - (player.x + crate::player::WIDTH * 0.5)).abs();
        assert!(dx < 20.0 * TILE_PX, "boss still {dx} px away");
    }

    #[test]
    fn attacking_kills_a_slime() {
        let w = ground();
        let mut s = Npc::spawn_slime(&w, 20).unwrap();
        for _ in 0..3 {
            assert!(s.hurt(10, 0.0));
            s.hurt_cd = 0.0;
        }
        assert_eq!(s.hp, 0);
    }
    fn bare(kind: NpcKind) -> Npc {
        Npc::new(kind, 100.0, 100.0, 1, 0.0)
    }

    #[test]
    fn slime_frames_cycle_every_eight_steps_and_speed_up_while_moving() {
        // NPC.cs:70801-70826: one counter tick per step, +1 when num2 > 0.
        let mut s = bare(NpcKind::Slime);
        for _ in 0..7 {
            s.find_frame();
        }
        assert_eq!(s.frame, 0);
        s.find_frame();
        assert_eq!(s.frame, 1, "8 idle steps advance one frame");
        // Moving (num2 == 1) doubles the tick rate: 4 steps to the next frame.
        s.vx = 1.0;
        for _ in 0..3 {
            s.find_frame();
        }
        assert_eq!(s.frame, 1);
        s.find_frame();
        assert_eq!(s.frame, 0, "moving ticks twice per step: 4 steps advance");
        // The strip wraps over Main.npcFrameCount[1] = 2 frames.
        for _ in 0..16 {
            s.find_frame();
        }
        assert_eq!(s.frame, 0);
    }

    #[test]
    fn zombie_walk_band_is_zero_one_two_one_by_accumulated_speed() {
        // NPC.cs:76324-76389: the counter accumulates |velocity.X| and the
        // frame follows the bands <8 -> 0, <16 -> 1, <24 -> 2, <32 -> 1; at 32
        // the counter restarts WITHOUT touching the frame (it stays 1 until
        // the next tick re-enters the first band).
        let mut z = bare(NpcKind::Zombie);
        z.vx = 0.9; // aligned with facing 1
        z.frame_counter = 0.0;
        z.frame = 0;
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 1, "9 ticks = 8.1 accumulated: second band");
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 2, "18 ticks = 16.2: third band");
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 1, "27 ticks = 24.3: fourth band");
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 1, "36 ticks = 32.4: restart keeps the frame");
        z.find_frame();
        assert_eq!(z.frame, 0, "the next tick re-enters the first band");
    }

    #[test]
    fn zombie_holds_the_lurch_frame_when_airborne_or_backpedalling() {
        // NPC.cs:76334-76342.
        let mut z = bare(NpcKind::Zombie);
        z.vy = 0.1;
        z.find_frame();
        assert_eq!(z.frame, 2);
        z.vy = 0.0;
        z.facing = 1;
        z.vx = -0.5; // moving against the facing
        z.find_frame();
        assert_eq!(z.frame, 2);
        z.vx = 0.0;
        z.find_frame();
        assert_eq!(z.frame, 0, "standing still is the idle frame");
    }

    #[test]
    fn king_slime_snaps_to_four_five_in_the_air_and_cycles_zero_to_three_on_ground() {
        // NPC.cs:73380-73414: airborne snaps to frame 4, holds it 4 steps,
        // then frame 5; landing returns to 4 and the grounded cycle advances
        // every 8 steps with the wrap `>= 4 -> 0`, so frames 4/5 never show
        // while grounded.
        let mut k = bare(NpcKind::KingSlime);
        k.vy = -0.1; // leaving the ground
        k.find_frame();
        assert_eq!(k.frame, 4, "airborne snaps straight to frame 4");
        for _ in 0..3 {
            k.find_frame();
        }
        assert_eq!(k.frame, 5, "after 4 airborne steps the squash frame shows");
        k.vy = 0.0; // landed
        k.find_frame();
        assert_eq!(k.frame, 4, "landing returns to frame 4, counter reset");
        // The landing squash (4) is itself displayed while grounded until the
        // first advance; frames 0..=3 then cycle. Only 5 is airborne-only.
        for _ in 0..64 {
            k.find_frame();
            assert!(
                k.frame <= 4,
                "grounded frames never show 5, got {}",
                k.frame
            );
        }
        // The grounded cycle must actually move through 0..=3.
        let mut k2 = bare(NpcKind::KingSlime);
        k2.vy = 0.0;
        let mut seen = [false; 4];
        for _ in 0..32 {
            k2.find_frame();
            seen[k2.frame as usize] = true;
        }
        assert!(
            seen.iter().all(|&s| s),
            "one 32-step cycle covers frames 0..=3"
        );
    }
}
