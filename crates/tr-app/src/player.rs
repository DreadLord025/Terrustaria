//! Player physics and collision, modelled on `Player.cs`.
//!
//! Core movement: gravity, acceleration/friction, jump, 1-tile auto-step, axis-separated
//! AABB collision against solid tiles, simple water buoyancy and ground-slope support
//! (slopes 1/2) and half bricks/ceiling slopes during movement. Survival follows
//! `Player.cs`: a 200-tick breath meter (1 unit every 7
//! ticks underwater, 2 damage once empty), fall damage over 25 tiles, lava damage and
//! out-of-combat health regeneration. Half bricks and ceiling slopes still count as full
//! cells.

use crate::registry;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World};

use crate::buffs::BuffSlots;
use crate::collision;
use crate::regen::{RegenInputs, RegenState, update_life_regen, update_mana_regen};

pub const WIDTH: f32 = 20.0;
pub const HEIGHT: f32 = 42.0;

const GRAVITY: f32 = 0.4;
const MAX_FALL: f32 = 10.0;
/// `Player.jumpSpeed` / `Player.jumpHeight`: holding the jump key keeps the
/// upward velocity at `JUMP_SPEED` for `JUMP_HEIGHT` ticks, so tap and hold jumps
/// reach different heights.
const JUMP_SPEED: f32 = 5.01;
const JUMP_HEIGHT: i32 = 15;
/// `Player.breathMax`; `breath` is in ticks.
pub const BREATH_MAX: i32 = 200;
/// `Player.breathCDMax`: ticks between breath decrements.
const BREATH_CD: i32 = 7;

#[derive(Default, Clone, Copy)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub down: bool,
}

/// The `WetCollision` movement speed per liquid (`Player.cs:28168-28171`):
/// water 0.5, lava 0.5, honey 0.25, shimmer 0.375, dry 1.0.
pub fn movement_speed_for(wet: bool, lava_wet: bool, honey_wet: bool, shimmer_wet: bool) -> f32 {
    if honey_wet {
        0.25
    } else if shimmer_wet {
        0.375
    } else if wet || lava_wet {
        0.5
    } else {
        1.0
    }
}

/// The in-liquid gravity (`Player.cs:27008`): `gravity / 3`.
pub fn liquid_gravity() -> f32 {
    GRAVITY / 3.0
}

/// The in-liquid fall cap, both ways (`Player.cs:27011-27017`):
/// `maxFallSpeed / 3`.
pub fn liquid_fall_cap() -> f32 {
    MAX_FALL / 3.0
}

#[derive(Clone, Debug)]
pub struct Player {
    // The class combat stats (`Player.cs:2430-2437`): every class starts at
    // crit 4 and a 1.0 damage multiplier; the equipment modifies them
    // (M3-EQUIP).
    pub melee_crit: i32,
    pub ranged_crit: i32,
    pub magic_crit: i32,
    pub melee_damage: f32,
    pub ranged_damage: f32,
    pub magic_damage: f32,
    /// `Player.dead`: the player is dead and waiting for the respawn timer
    /// (`Player.cs:38908`).
    pub dead: bool,
    /// `Player.deadTime`: the ticks spent dead (`Player.cs:16923`); gates the
    /// respawn-skip (90-tick lockout, `Player.cs:57111`).
    pub dead_time: i32,
    /// `Player.respawnTimer`: the ticks left until the respawn.
    pub respawn_timer: i32,
    /// The jump-input edge state for the dead-skip roll.
    prev_jump_input: bool,
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
    /// `Player.legFrame.Y / 56`: the current leg animation row (0..=19), driven by
    /// `PlayerFrame()`.
    pub leg_frame: u32,
    /// `Player.bodyFrame.Y / 56`: the current body animation row (0..=19). Mirrors the
    /// legs while walking; the app overrides it with the item-use rows during a swing.
    pub body_frame: u32,
    /// `Player.legFrameCounter`: accumulates `|velocity.X| * 1.3` while walking and
    /// advances a leg row every `8.0`.
    leg_frame_counter: f32,
    /// `Player.wet` — the head/body in WATER (`Player.cs:27728`).
    pub wet: bool,
    /// `Player.lavaWet` — in LAVA.
    pub lava_wet: bool,
    /// `Player.honeyWet` — in HONEY.
    pub honey_wet: bool,
    /// `Player.shimmerWet` — in SHIMMER.
    pub shimmer_wet: bool,
    /// The `WetCollision` movement-speed scale for this tick's position delta
    /// (`Player.cs:28168-28171`): water/lava 0.5, honey 0.25, shimmer 0.375,
    /// dry 1.0.
    pub liquid_step: f32,
    pub in_water: bool,
    /// Remaining ticks of the held-jump boost (`Player.jump`).
    jump: i32,
    /// Jump was held on the previous fixed step, for press-edge detection.
    jump_held: bool,
    pub spawn_x: f32,
    pub spawn_y: f32,
    pub hp: i32,
    pub max_hp: i32,
    /// `Player.statMana` / `Player.statManaMax` (`Player.cs:1930-1932`). A new
    /// character starts at `20/20` (`UICharacterCreation.cs:1721`; the default max
    /// is also 20 in `Player.Initialize`, `Player.cs:56075`).
    pub mana: i32,
    pub mana_max: i32,
    /// The 44 buff slots and `buffImmune` — `Player.buffType`/`buffTime`/`buffImmune`
    /// (`Player.cs:1510-1514`), driven by the `Player.AddBuff` family (`buffs.rs`).
    pub buffs: BuffSlots,
    pub hurt_cd: f32,
    /// Remaining breath in ticks (`Player.breath`).
    pub breath: i32,
    breath_cd: i32,
    /// The life/mana regen accumulators (`Player.lifeRegen`/`lifeRegenTime`/
    /// `lifeRegenCount`/`manaRegen*`, `Player.cs:1930-1946`) — `regen.rs`.
    pub regen: RegenState,
    /// `Player.fallStart` (`Player.cs:2882`): the tile row the fall started on,
    /// int-truncated like the original's `(int)(position.Y / 16f)`.
    fall_start: i32,
    /// `Player.extraFall` (`Player.cs:2726`): tiles added to the 25-tile safe
    /// fall. Nothing grants it yet (frog-gear sources = M3-EQUIP-3), but the
    /// reset-each-tick shape is the vanilla one.
    extra_fall: i32,
    /// `Player.noFallDmg` (`Player.cs:2766`): the horseshoe/balloon family
    /// (M3-EQUIP-3). Always false until the accessory system lands.
    no_fall_dmg: bool,
    /// `Player.hasWings` (`Player.cs:1336`): any wing equipped (M3-EQUIP-3).
    has_wings: bool,
    /// `Player.lastStoned` (`Player.cs`): the petrify-transition detector.
    last_stoned: bool,
    /// `Player.maxRunSpeed` / `runAcceleration` / `runSlowdown` /
    /// `accRunSpeed` (`Player.cs:24364-24371`): the per-tick horizontal
    /// defaults the `HorizontalMovement` branches and the floor modifiers ride.
    max_run_speed: f32,
    run_acceleration: f32,
    run_slowdown: f32,
    acc_run_speed: f32,
    /// `Player.moveSpeed` / `strongestMoveSpeedDebuff` (`Player.cs:2464,2462`):
    /// 1.0 until accessories/debuffs land (M3-EQUIP-3, M2-BUFF-4).
    move_speed: f32,
    strongest_move_speed_debuff: f32,
    /// `Player.iceSkate` / `desertBoots` (`Player.cs:1088,2256`): Ice Skates /
    /// Dunerider Boots (M3-EQUIP-3 — no accessory system yet).
    ice_skate: bool,
    desert_boots: bool,
    /// The `FloorVisuals` flags (`Player.cs:23708-23743`), sampled after the
    /// tick's movement — one tick stale relative to the movement block,
    /// exactly like the original's call order (`:26500` vs `:28271`).
    sticky: bool,
    slippy: bool,
    slippy2: bool,
    powerrun: bool,
    running_on_sand: bool,
    /// The liquid displacement fraction `Player.WetCollision` integrates free
    /// axes with (`Player.cs:23436-23451`): water/lava 0.5, honey 0.25
    /// (`Player.cs:28168-28195`), dry 1.0.
    liquid_move_speed: f32,
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
            dead: false,
            dead_time: 0,
            respawn_timer: 0,
            prev_jump_input: false,
            melee_crit: 4,
            ranged_crit: 4,
            magic_crit: 4,
            melee_damage: 1.0,
            ranged_damage: 1.0,
            magic_damage: 1.0,
            x: sx,
            y: sy,
            prev_x: sx,
            prev_y: sy,
            vx: 0.0,
            vy: 0.0,
            on_ground: false,
            facing: 1,
            leg_frame: 0,
            body_frame: 0,
            leg_frame_counter: 0.0,
            wet: false,
            lava_wet: false,
            honey_wet: false,
            shimmer_wet: false,
            liquid_step: 1.0,
            in_water: false,
            jump: 0,
            jump_held: false,
            spawn_x: sx,
            spawn_y: sy,
            hp: 100,
            max_hp: 100,
            mana: 20,
            mana_max: 20,
            buffs: BuffSlots::default(),
            hurt_cd: 0.0,
            breath: BREATH_MAX,
            breath_cd: 0,
            regen: RegenState::default(),
            fall_start: (sy / TILE_PX) as i32,
            extra_fall: 0,
            no_fall_dmg: false,
            has_wings: false,
            last_stoned: false,
            max_run_speed: 3.0,
            run_acceleration: 0.08,
            run_slowdown: 0.2,
            acc_run_speed: 3.0,
            move_speed: 1.0,
            strongest_move_speed_debuff: 1.0,
            ice_skate: false,
            desert_boots: false,
            sticky: false,
            slippy: false,
            slippy2: false,
            powerrun: false,
            running_on_sand: false,
            liquid_move_speed: 1.0,
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
        self.fall_start = (y / TILE_PX) as i32;
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
        self.leg_frame = 0;
        self.body_frame = 0;
        self.leg_frame_counter = 0.0;
        self.hp = self.max_hp;
        self.hurt_cd = 0.0;
        self.breath = BREATH_MAX;
        self.breath_cd = 0;
        self.regen = RegenState::default();
        self.fall_start = (self.y / TILE_PX) as i32;
        self.wet = false;
        self.lava_wet = false;
        self.honey_wet = false;
        self.shimmer_wet = false;
        self.in_lava = false;
        self.gfx_off_y = 0.0;
        self.prev_gfx_off_y = 0.0;
    }

    /// `Player.KillMe`'s respawn setup (`Player.cs:38908-38910`): the dead
    /// flag, `deadTime` reset and `respawnTimer = GetRespawnTime(pvp)`.
    pub fn start_death(&mut self, boss_near: bool, expert: bool) {
        self.dead = true;
        self.dead_time = 0;
        self.respawn_timer = Player::get_respawn_time(boss_near, expert);
    }

    /// `Player.GetRespawnTime` (`Player.cs:39060-39082`): 600 base, +600 when
    /// a boss hinders the respawn, x1.5 in expert. The FTW-multiplier x2 and
    /// the master-mode path stay with M5-SEED-1 / the master difficulty.
    pub fn get_respawn_time(boss_near: bool, expert: bool) -> i32 {
        let mut time = 600;
        if boss_near {
            time += 600;
        }
        if expert {
            time = (time as f64 * 1.5) as i32;
        }
        time
    }

    /// `NPC.AnyBossHindersRespawnTime` shape (`NPC.cs`): a boss within the
    /// 4000 px Manhattan range of the death spot. The King Slime is the
    /// ported boss.
    pub fn boss_hinders(&self, npcs: &[crate::npc::Npc]) -> bool {
        let (cx, cy) = self.center();
        npcs.iter().any(|n| {
            n.kind == crate::npc::NpcKind::KingSlime && {
                let (nx, ny) = n.center();
                (cx - nx).abs() + (cy - ny).abs() < 4000.0
            }
        })
    }

    /// `Player.AllowsRespawnTimerSkip` (`Player.cs:38962-38987`): after the
    /// 90-tick lockout, with no boss hindering and no danger NPCs nearby.
    /// `ghost`/`pvpDeath`/event branches are unreachable in the port.
    pub fn allows_respawn_timer_skip(&self, boss_near: bool, danger_near: bool) -> bool {
        !(self.ghost_like() || self.dead_time < 90 || boss_near || danger_near)
    }

    /// The port has no ghosts (hardcore) or PVP deaths; both gates are false.
    fn ghost_like(&self) -> bool {
        false
    }

    /// `Player.UpdateDead` (`Player.cs:16919-17070`): `deadTime++`, the
    /// respawn decrement clamped to `respawnTimerMax` (3600), and the
    /// jump-press skip while `AllowsRespawnTimerSkip`. Returns true when the
    /// player should respawn now.
    pub fn update_dead(&mut self, jump: bool, boss_near: bool, danger_near: bool) -> bool {
        self.dead_time += 1;
        self.respawn_timer = (self.respawn_timer - 1).clamp(0, 3600);
        let jump_edge = jump && !self.prev_jump_input;
        self.prev_jump_input = jump;
        let can_skip = self.allows_respawn_timer_skip(boss_near, danger_near);
        if can_skip && jump_edge && self.respawn_timer > 0 {
            return true;
        }
        self.respawn_timer <= 0
    }

    /// `Player.Spawn(ReviveFromDeath)`: the dead flags clear, then the
    /// standard spawn-point respawn.
    pub fn revive(&mut self) {
        self.dead = false;
        self.dead_time = 0;
        self.respawn_timer = 0;
        self.respawn();
    }

    /// Contact damage / future combat. Returns true when the hit was accepted.
    pub fn hurt(&mut self, amount: i32, from_x: f32) -> bool {
        self.hurt_inner(amount, from_x, true)
    }

    /// The shared hurt body. `knock_back` mirrors the original's
    /// `hitDirection != 0` gate (`Player.cs:38519`): the fall-damage and
    /// petrify calls pass hitDirection 0 and skip the knock entirely (the
    /// fallStart reset inside that branch rides with it).
    fn hurt_inner(&mut self, amount: i32, from_x: f32, knock_back: bool) -> bool {
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        self.hp = (self.hp - amount).max(0);
        // `Player.Hurt`'s immunity window (`Player.cs:38385`): 40 ticks
        // for normal damage (no long-invincibility source yet), 20 when
        // the damage is exactly 1; PVP 8 is unreachable in single-player.
        self.hurt_cd = tr_core::damage::iframes_for(amount, false, false) as f32 / 60.0;
        // `Player.Hurt` resets the natural-ramp timer (`Player.cs:38395`).
        self.regen.life_regen_time = 0.0;
        if knock_back {
            let dir = if self.x + WIDTH * 0.5 >= from_x {
                1.0
            } else {
                -1.0
            };
            // The fixed contact knockback (`Player.cs` Hurt: `4.5 * dir` with
            // `-3.5` vertical unless noKnockback).
            self.vx = dir * 4.5;
            self.vy = -3.5;
            self.on_ground = false;
        }
        true
    }

    /// The fall-damage landing hit (`Player.cs:25361-25370`): the normal branch
    /// forces `immune = false` BEFORE the Hurt — the hit lands through any
    /// iframes — and hitDirection 0 means no knockback and no fallStart reset.
    fn hurt_fall(&mut self, amount: i32) -> bool {
        self.hurt_cd = 0.0;
        let cx = self.x + WIDTH * 0.5;
        self.hurt_inner(amount, cx, false)
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + WIDTH * 0.5, self.y + HEIGHT * 0.5)
    }

    /// `Player.GetWeaponDamageMultiplier` (`Player.cs:53386`) narrowed to
    /// the ported classes: the classless items pass the 1f default.
    pub fn weapon_damage_multiplier(&self, class: registry::DamageClass) -> f32 {
        match class {
            registry::DamageClass::Melee => self.melee_damage,
            registry::DamageClass::Ranged => self.ranged_damage,
            registry::DamageClass::Magic => self.magic_damage,
            _ => 1.0,
        }
    }

    /// The class crit source (`Player.meleeCrit`/`rangedCrit`/`magicCrit`,
    /// `Player.cs:2430-2432`; base 4 per class).
    pub fn class_crit(&self, class: registry::DamageClass) -> i32 {
        match class {
            registry::DamageClass::Melee => self.melee_crit,
            registry::DamageClass::Ranged => self.ranged_crit,
            registry::DamageClass::Magic => self.magic_crit,
            _ => 0,
        }
    }

    /// Direct health loss with no knockback or i-frames, used by drowning and fall
    /// damage (`statLife -= ...` in `Player.cs`).
    fn lose_health(&mut self, amount: i32) {
        if self.hp <= 0 {
            return;
        }
        self.hp = (self.hp - amount).max(0);
        self.hurt_cd = self.hurt_cd.max(0.8);
        // No ramp reset here: the C# direct `statLife -=` paths (drowning, fall
        // damage) leave `lifeRegenTime` untouched.
    }

    /// One fixed 60 Hz step.
    pub fn update(&mut self, world: &World, input: PlayerInput) {
        // `Player.Update` → `UpdateBuffs(i)` (`Player.cs:24535`): the timer head runs
        // early in the tick. Nothing grants buffs yet, so this only decrements.
        self.buffs.update_buffs_tick();
        self.prev_x = self.x;
        self.prev_y = self.y;
        self.prev_gfx_off_y = self.gfx_off_y;
        self.hurt_cd = (self.hurt_cd - 1.0 / 60.0).max(0.0);
        let (cx, cy) = self.center();
        let mid = world.tile((cx / TILE_PX) as i32, (cy / TILE_PX) as i32);
        // `Collision.WetCollision` / `LavaCollision` set the four wet flags
        // (`Player.cs:27728-27910`); the mid-tile liquid kind picks them.
        let liquid = mid.filter(|t| t.liquid > 0).map(|t| t.liquid_kind());
        self.wet = liquid == Some(tr_core::tile::LiquidKind::Water);
        self.lava_wet = liquid == Some(tr_core::tile::LiquidKind::Lava);
        self.honey_wet = liquid == Some(tr_core::tile::LiquidKind::Honey);
        self.shimmer_wet = liquid == Some(tr_core::tile::LiquidKind::Shimmer);
        self.in_water = self.wet || self.honey_wet;
        self.in_lava = self.lava_wet;

        // The fall origin is only reset post-movement now: the landing/rest
        // block (`velocity.Y == 0`, `Player.cs:25383`) and the continuous
        // condition (`Player.cs:25379`). The old pre-movement `vy <= 0` reset
        // measured falls from every upward apex — the vanilla code never does.

        // `Player.cs:24364-24371`: the per-tick movement defaults;
        // `moveSpeed` multiplies acceleration and the primary gate while
        // `accRunSpeed` takes only the debuff (`Player.cs:26004-26010`).
        self.max_run_speed = 3.0;
        self.run_acceleration = 0.08;
        self.run_slowdown = 0.2;
        self.acc_run_speed = self.max_run_speed;
        self.move_speed = 1.0;
        self.strongest_move_speed_debuff = 1.0;
        self.move_speed *= self.strongest_move_speed_debuff;
        self.run_acceleration *= self.move_speed;
        self.max_run_speed *= self.move_speed;
        self.acc_run_speed *= self.strongest_move_speed_debuff;
        // `Player.cs:26522-26579`: the floor-tile modifiers, one else-if chain.
        // The flags come from the previous tick's `FloorVisuals` sample — one
        // tick stale, like the original's call order. The empress-brooch /
        // Magiluminescence / shadow-armor boosts land with M3-EQUIP-3.
        if self.sticky {
            self.max_run_speed *= 0.25;
            self.run_acceleration *= 0.25;
            self.run_slowdown *= 2.0;
            self.vx = self.vx.clamp(-self.max_run_speed, self.max_run_speed);
        } else if self.powerrun {
            self.max_run_speed *= 3.5;
            self.run_slowdown *= 2.0;
        } else if self.running_on_sand && self.desert_boots {
            let n = 1.75;
            self.max_run_speed *= n;
            self.acc_run_speed *= n;
            self.run_acceleration *= n;
            self.run_slowdown *= n;
        } else if self.slippy2 {
            self.run_acceleration *= 0.6;
            self.run_slowdown = 0.0;
            if self.ice_skate {
                self.run_acceleration *= 3.5;
                self.max_run_speed *= 1.25;
            }
        } else if self.slippy {
            self.run_acceleration *= 0.7;
            if self.ice_skate {
                self.run_acceleration *= 3.5;
                self.max_run_speed *= 1.25;
            } else {
                self.run_slowdown *= 0.1;
            }
        }
        let dir = (input.right as i32 - input.left as i32) as f32;
        if dir != 0.0 {
            self.facing = dir as i32;
        }
        self.horizontal_movement(dir);

        let in_liquid = self.wet || self.lava_wet || self.honey_wet || self.shimmer_wet;
        // `Player.cs:27008-27017`: in a liquid the gravity is `gravity / 3`
        // and the fall cap is `maxFallSpeed / 3`, applied both ways (the
        // upward cap keeps the swim under the surface).
        let gravity = if in_liquid { liquid_gravity() } else { GRAVITY };
        let fall_cap = if in_liquid {
            liquid_fall_cap()
        } else {
            MAX_FALL
        };
        self.vy = (self.vy + gravity).min(fall_cap);
        self.vy = self.vy.max(if in_liquid {
            -(MAX_FALL / 3.0)
        } else {
            -MAX_FALL
        });
        // `WetCollision`'s `movementSpeed` (`Player.cs:28168-28171` + :23436):
        // the position delta scales by the liquid speed — water/lava 0.5,
        // honey 0.25, shimmer 0.375. The scaled step feeds the collision; the
        // velocity itself keeps integrating (the exit speed is preserved).
        self.liquid_step =
            movement_speed_for(self.wet, self.lava_wet, self.honey_wet, self.shimmer_wet);

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

        self.move_vanilla(world, input.down, gravity);

        // `Player.cs:25219-25224`: the tick petrification lands, a flat hit —
        // the difficulty multiplier is 1 (classic) until M6-CREATIVE lands.
        let stoned = self.buffs.find_buff_index(156).is_some();
        if stoned && !self.last_stoned {
            let cx = self.x + WIDTH * 0.5;
            self.hurt_inner(20, cx, false);
        }
        self.last_stoned = stoned;

        // `Player.cs:25293-25385`: the fall-damage block — the whole thing runs
        // whenever the player has come to rest this tick (`velocity.Y == 0`).
        if self.vy == 0.0 {
            // `num10 = 25 + extraFall` (`Player.cs:25301`); the mount addendum
            // (`Player.cs:25304`) waits for M2-MOUNT-1.
            let threshold = 25 + self.extra_fall;
            // `num11` is int-truncated on both ends (`Player.cs:25306`).
            let mut fell = (self.y / TILE_PX) as i32 - self.fall_start;
            // The flying-minecart/bunny/dash zeroes (`Player.cs:25307-25323`)
            // have no systems here yet; the cloud/soft-tile scan does
            // (`Player.cs:25325-25345`): the tile row under the feet across
            // the hitbox's x-extent.
            let tx0 = (self.x / TILE_PX) as i32;
            let tx1 = ((self.x + WIDTH) / TILE_PX) as i32;
            let ty = ((self.y + HEIGHT + 1.0) / TILE_PX) as i32;
            for tx in tx0..=tx1 {
                if let Some(t) = world.tile(tx, ty) {
                    if t.active()
                        && (tr_core::tiledata::SET_CLOUDS.has(t.kind)
                            || t.kind == 666
                            || (t.kind == 19 && (t.frame_y / 18) == 49))
                    {
                        fell = 0;
                        break;
                    }
                }
            }
            if stoned {
                // `Player.cs:25347-25359`: petrified — a fixed 2-tile grace and
                // 20 damage per tile; extraFall and the mount multiplier are
                // ignored, and the immunity clears AFTER the hit.
                let dmg = (fell - 2) * 20;
                if dmg > 0 {
                    let cx = self.x + WIDTH * 0.5;
                    self.hurt_inner(dmg, cx, false);
                    self.hurt_cd = 0.0;
                }
            } else if (fell > threshold) && !self.no_fall_dmg && !self.has_wings {
                // `Player.cs:25360-25374`: `(fell - threshold) * 10` damage with
                // hitDirection 0 — no knockback — and `immune = false` forced
                // BEFORE the Hurt so the hit lands through any iframes.
                self.hurt_fall((fell - threshold) * 10);
            }
            self.fall_start = (self.y / TILE_PX) as i32;
        }
        // `Player.cs:25379-25382`: the continuous reset — jumping, liquid, the
        // slow-fall/rocket/sky-gravity/tongued terms (the four systems are not
        // ported yet) keep the fall origin at the current row.
        if self.jump > 0 || self.in_water {
            self.fall_start = (self.y / TILE_PX) as i32;
        }

        // `Player.cs:28269-28272`: `FloorVisuals` after this tick's movement —
        // the movement block reads the flags next tick (one tick stale).
        if self.vy == 0.0 {
            self.floor_visuals(world);
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

        // The immune purge runs near the end of the original update, after movement
        // and effects (`Player.cs:26012-26018`).
        self.buffs.purge_immune_buffs();

        self.update_frames();
    }

    /// `PlayerFrame()` (legs + the locomotion part of the body). The original runs this
    /// every tick after movement:
    /// - idle: legs and body row `0` (the rest pose);
    /// - airborne (`velocity.Y != 0`): row `5` (the jump/fall pose);
    /// - walking (`velocity.X != 0`): legs advance one row per `8.0` of
    ///   `legFrameCounter += |velocity.X| * 1.3`, cycling the walk band rows `7..=19`,
    ///   and the body copies the leg row.
    ///
    /// The wings/mount/pulley/hold-style branches of the original are not modelled yet,
    /// and item-use body frames are applied by the app via [`Player::apply_item_use_frame`]
    /// while a swing is active.
    fn update_frames(&mut self) {
        if self.vy != 0.0 {
            self.leg_frame_counter = 0.0;
            self.leg_frame = 5;
        } else if self.vx != 0.0 {
            self.leg_frame_counter += self.vx.abs() * 1.3;
            while self.leg_frame_counter > 8.0 {
                self.leg_frame_counter -= 8.0;
                self.leg_frame += 1;
            }
            if self.leg_frame < 7 {
                self.leg_frame = 19;
            } else if self.leg_frame > 19 {
                self.leg_frame = 7;
            }
        } else {
            self.leg_frame_counter = 0.0;
            self.leg_frame = 0;
        }

        if self.vy != 0.0 {
            self.body_frame = 5;
        } else if self.vx != 0.0 {
            self.body_frame = self.leg_frame;
        } else {
            self.body_frame = 0;
        }
    }

    /// `PlayerFrame()` item-use branch for `useStyle == 1` (blocks and other placeables):
    /// the body steps rows `1 -> 2 -> 3` as `itemAnimation` runs out. `remaining` is
    /// `itemAnimation / itemAnimationMax` (1 at the start of the swing, 0 at the end).
    pub fn apply_item_use_frame(&mut self, remaining: f32) {
        self.body_frame = if remaining < 0.333 {
            3
        } else if remaining < 0.666 {
            2
        } else {
            1
        };
    }

    /// `Player.HorizontalMovement` (`Player.cs:19772-20088`): the primary
    /// acceleration gates on `maxRunSpeed`; between `maxRunSpeed` and
    /// `accRunSpeed` (the boot top speeds) acceleration continues at 0.2x —
    /// ground-only here, the wings/flying-mount terms land with PHYS-5/M2-
    /// MOUNT-1, and `dashDelay >= 0` always holds until M2-PHYS-4. No input
    /// decays by `runSlowdown` on the ground and half of it in the air. There
    /// is no hard clamp — the decay bounds the speed (knockback rides out).
    fn horizontal_movement(&mut self, dir: f32) {
        if dir < 0.0 {
            if self.vx > -self.max_run_speed {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                }
                self.vx -= self.run_acceleration;
            } else if self.vx > -self.acc_run_speed && self.vy == 0.0 {
                if self.vx > self.run_slowdown {
                    self.vx -= self.run_slowdown;
                }
                self.vx -= self.run_acceleration * 0.2;
            }
        } else if dir > 0.0 {
            if self.vx < self.max_run_speed {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                }
                self.vx += self.run_acceleration;
            } else if self.vx < self.acc_run_speed && self.vy == 0.0 {
                if self.vx < -self.run_slowdown {
                    self.vx += self.run_slowdown;
                }
                self.vx += self.run_acceleration * 0.2;
            }
        } else if self.vy == 0.0 {
            if self.vx > self.run_slowdown {
                self.vx -= self.run_slowdown;
            } else if self.vx < -self.run_slowdown {
                self.vx += self.run_slowdown;
            } else {
                self.vx = 0.0;
            }
        } else if self.vx > self.run_slowdown * 0.5 {
            self.vx -= self.run_slowdown * 0.5;
        } else if self.vx < -self.run_slowdown * 0.5 {
            self.vx += self.run_slowdown * 0.5;
        } else {
            self.vx = 0.0;
        }
    }

    /// `Player.FloorVisuals` (`Player.cs:23708-23743`) + `GetFloorTile`
    /// (`:23755-23783`): the tile row at the feet, center column first, then
    /// the neighbors, solid tiles only. The dust visuals (MakeFloorDust) and
    /// the slope dust carve-out stay with the particle tasks.
    fn floor_visuals(&mut self, world: &World) {
        let x = ((self.x + WIDTH * 0.5) / TILE_PX) as i32;
        let y = ((self.y + HEIGHT) / TILE_PX) as i32;
        let mut kind = None;
        for tx in [x, x - 1, x + 1] {
            if let Some(t) = world.tile(tx, y) {
                if t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind) {
                    kind = Some(t.kind);
                    break;
                }
            }
        }
        let Some(kind) = kind else {
            self.sticky = false;
            self.slippy = false;
            self.slippy2 = false;
            self.powerrun = false;
            self.running_on_sand = false;
            return;
        };
        // `Player.cs:23727-23731`: cobweb 229, the IceSkateSlippery set
        // (`TileID.cs:481`), FrozenSlimeBlock 197, Asphalt 198, and the sand
        // conversions (`TileID.cs:92-94`).
        self.sticky = kind == 229;
        self.slippy = tr_core::tiledata::SET_ICE_SKATE_SLIPPERY.has(kind);
        self.slippy2 = kind == 197;
        self.powerrun = kind == 198;
        self.running_on_sand = tr_core::tiledata::SET_CONVERSION_SAND.has(kind)
            || tr_core::tiledata::SET_CONVERSION_SANDSTONE.has(kind)
            || tr_core::tiledata::SET_CONVERSION_HARDENED_SAND.has(kind);
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

        // `Player.Update` → `UpdateLifeRegen` / `UpdateManaRegen`
        // (`Player.cs:25977,25995`). The DoT/aura/item/grapple inputs stay at
        // their defaults until those systems land (`regen.rs`); the hp gate
        // stands in for the original's dead-player check (M2-DEATH).
        if self.hp > 0 {
            let in_ = RegenInputs {
                max_life: self.max_hp,
                max_mana: self.mana_max,
                moving_x: self.vx != 0.0,
                standing_still: self.vx.abs() < 0.05 && self.vy.abs() < 0.05,
                ..RegenInputs::default()
            };
            update_life_regen(&mut self.regen, &in_, &mut self.hp);
            update_mana_regen(&mut self.regen, &in_, &mut self.mana);
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
        // still carries this tick's gravity), then resolves the AABB. The wet
        // path never steps — `WetCollision` goes straight to `TileCollision`
        // (`Player.cs:23443`), the step helpers are `DryCollision` internals.
        let dry = self.liquid_step == 1.0;
        if dry && vel[1] == gravity {
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
        if dry && vel[1] >= gravity && !down {
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
        // `Player.WetCollision` (`Player.cs:23436-23451`): the collision sees
        // the unscaled velocity; the free axes then integrate at the liquid
        // fraction (`liquid_step` — water/lava 0.5, honey 0.25, shimmer 0.375)
        // while an axis the collision overrode keeps its full velocity, so
        // momentum survives the water.
        let pre = vel;
        vel = tc.vel;
        let ms = self.liquid_step;
        pos[0] += if vel[0] != pre[0] { vel[0] } else { vel[0] * ms };
        pos[1] += if vel[1] != pre[1] { vel[1] } else { vel[1] * ms };
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
        h ^= self.leg_frame << 8;
        h ^= self.body_frame << 16;
        // The draw bakes `gfxOffY` into the sprite's screen position
        // (`LegacyPlayerRenderer.cs:457,468`), so its ease after a step must
        // invalidate the cached range or the offset animation freezes.
        h ^= self.gfx_off_y.to_bits().wrapping_mul(0x27D4_EB2F);
        h
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
    #[test]
    fn movement_speed_per_liquid() {
        // Player.cs:28168-28171: water/lava 0.5, honey 0.25, shimmer 0.375,
        // dry 1.0.
        assert_eq!(movement_speed_for(true, false, false, false), 0.5);
        assert_eq!(movement_speed_for(false, true, false, false), 0.5);
        assert_eq!(movement_speed_for(false, false, true, false), 0.25);
        assert_eq!(movement_speed_for(false, false, false, true), 0.375);
        assert_eq!(movement_speed_for(false, false, false, false), 1.0);
    }

    #[test]
    fn in_liquid_gravity_and_cap_are_thirds() {
        // Player.cs:27008-27017: gravity / 3 (0.4 -> 0.1333) and the
        // both-ways cap at maxFallSpeed / 3 (10 -> 3.333).
        assert!((liquid_gravity() - 0.4 / 3.0).abs() < 1e-6);
        assert!((liquid_fall_cap() - 10.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn respawn_time_base_boss_and_expert() {
        // Player.cs:39060-39082: 600 base, +600 boss-near, x1.5 expert.
        assert_eq!(Player::get_respawn_time(false, false), 600);
        assert_eq!(Player::get_respawn_time(true, false), 1200);
        assert_eq!(Player::get_respawn_time(false, true), 900);
        assert_eq!(Player::get_respawn_time(true, true), 1800);
    }

    #[test]
    fn update_dead_counts_down_and_revives_at_zero() {
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(false, false);
        assert!(p.dead);
        assert_eq!(p.respawn_timer, 600);
        // 600 ticks of the dead state (no jump) -> the timer drains to 0 and
        // the last tick reports the respawn.
        let mut revived_at = None;
        for tick in 0..700 {
            if p.update_dead(false, false, false) {
                revived_at = Some(tick);
                break;
            }
        }
        assert_eq!(revived_at, Some(599));
        p.revive();
        assert!(!p.dead);
        assert_eq!(p.respawn_timer, 0);
    }

    #[test]
    fn respawn_timer_clamps_at_3600() {
        // Player.respawnTimerMax = 3600 (Player.cs:57109): a boss-near expert
        // death (1800) halved by nothing still clamps; the clamp matters for
        // the >3600 totals (the FTW x2 stays out). A forced high timer holds.
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(true, true);
        p.respawn_timer = 4000; // the save-load can exceed the cap
        p.update_dead(false, false, false);
        // The clamp catches the decrement result (Player.cs:17049).
        assert_eq!(p.respawn_timer, 3600);
    }

    #[test]
    fn the_skip_needs_the_90_tick_lockout_and_a_jump_edge() {
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(false, false);
        // Inside the 90-tick lockout: no skip even on the jump edge (the
        // held jump's edge is consumed while the lockout runs).
        for _ in 0..89 {
            assert!(!p.update_dead(true, false, false));
        }
        // The lockout expired but the held jump has no fresh edge; a release
        // and a new press do.
        assert!(!p.update_dead(true, false, false));
        assert!(!p.update_dead(false, false, false));
        assert!(p.update_dead(true, false, false));
    }

    #[test]
    fn the_skip_is_blocked_while_a_boss_hinders() {
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(false, false);
        for _ in 0..95 {
            p.update_dead(false, false, false);
        }
        assert!(!p.allows_respawn_timer_skip(true, false));
        assert!(p.allows_respawn_timer_skip(false, false));
    }

    #[test]
    fn the_jump_edge_is_required_for_the_skip() {
        // Holding jump through the lockout does not skip on expiry; a fresh
        // press does (the JustBecameTrue semantics of PlayerInput.Triggers).
        let mut p = Player::spawn(&World::new("t".into(), 64, 800));
        p.start_death(false, false);
        for _ in 0..95 {
            p.update_dead(true, false, false);
        }
        // The held jump consumed its edge at the lockout expiry...
        assert!(!p.update_dead(true, false, false));
        // ...release and press again: the edge fires.
        p.update_dead(false, false, false);
        assert!(p.update_dead(true, false, false));
    }

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

    /// Drops the player from `from_tile` onto the ground_world surface (top
    /// row 60) and simulates until rest. Returns (hp lost, landed tile row).
    fn drop_and_settle(w: &World, from_tile: i32) -> (i32, i32) {
        let mut p = Player::spawn(w);
        let hp0 = p.hp;
        p.y = (from_tile * 16) as f32;
        p.fall_start = from_tile;
        for _ in 0..600 {
            p.update(w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        (hp0 - p.hp, (p.y / TILE_PX) as i32)
    }

    /// A `ground_world` whose surface top row is `kind` (ice 161, cobweb 229,
    /// asphalt 198 — all solid in `TILE_SOLID`).
    fn floor_world(kind: u16) -> World {
        let mut w = ground_world();
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i].kind = kind;
        }
        w
    }

    /// Walks right for `ticks` with the input held, returning the final vx.
    fn walk(w: &World, ticks: u32) -> f32 {
        let mut p = Player::spawn(w);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..ticks {
            p.update(w, input);
        }
        p.vx
    }

    /// `Player.cs:26560-26570`: plain ice multiplies `runSlowdown` by 0.1 when
    /// nobody skates — releasing the key, the player slides ten times further
    /// than on dirt.
    #[test]
    fn ice_slides_without_skates() {
        let ice = floor_world(161);
        let mut p = Player::spawn(&ice);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..60 {
            p.update(&ice, input);
        }
        let vx0 = p.vx;
        for _ in 0..30 {
            p.update(&ice, PlayerInput::default());
        }
        let on_ice = p.vx;
        let mut q = Player::spawn(&ground_world());
        q.vx = vx0;
        for _ in 0..30 {
            q.update(&ground_world(), PlayerInput::default());
        }
        let on_dirt = q.vx;
        assert!(vx0 > 2.5, "should have accelerated, vx={vx0}");
        assert!(on_ice > 1.5, "ice keeps sliding: {on_ice}");
        assert!(
            on_dirt < 0.05,
            "dirt friction stops the player: {on_dirt}"
        );
    }

    /// `Player.cs:26562-26565`: with `iceSkate` the ice multiplies
    /// `runAcceleration` by 3.5 and `maxRunSpeed` by 1.25 — the top speed on
    /// ice exceeds the dirt one (3.0).
    #[test]
    fn ice_skates_raise_the_top_speed() {
        let ice = floor_world(161);
        let mut p = Player::spawn(&ice);
        p.ice_skate = true;
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..120 {
            p.update(&ice, input);
        }
        assert!(
            p.vx > 3.7 && p.vx < 4.0,
            "skate top speed ≈ 3.75, vx={}",
            p.vx
        );
        let dirt_top = walk(&ground_world(), 120);
        assert!(
            dirt_top > 2.9 && dirt_top < 3.2,
            "dirt top speed = 3, vx={dirt_top}"
        );
    }

    /// `Player.cs:26522-26534`: the cobweb multiplies speed 0.25x, slowdown
    /// 2x, and hard-clamps the current velocity to the reduced max.
    #[test]
    fn cobweb_slow_and_clamps() {
        let web = floor_world(229);
        let mut p = Player::spawn(&web);
        for _ in 0..300 {
            p.update(&web, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        // Standing on the web, the flags are live: an entry speed at
        // knockback strength clamps to the reduced max on the next tick.
        p.vx = 4.5;
        p.update(&web, PlayerInput::default());
        assert!(p.vx < 1.0, "web clamps and drags: vx={}", p.vx);
    }

    /// `Player.cs:23436-23451`: in water the free axes integrate at 0.5, in
    /// honey at 0.25 — momentum (vx) is untouched, the stride halves.
    #[test]
    fn water_halves_the_stride() {
        let mut w = ground_world();
        for y in 58..60 {
            for x in 0..64 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 200;
            }
        }
        let mut p = Player::spawn(&w);
        let input = PlayerInput {
            right: true,
            ..Default::default()
        };
        for _ in 0..90 {
            p.update(&w, input);
        }
        // Both runners are settled into their gait; measure the stride over
        // the next 30 ticks only.
        let x0 = p.x;
        let mut q = Player::spawn(&ground_world());
        for _ in 0..90 {
            q.update(&ground_world(), input);
        }
        let q0 = q.x;
        for _ in 0..30 {
            p.update(&w, input);
            q.update(&ground_world(), input);
        }
        let ratio = (p.x - x0) / (q.x - q0);
        assert!(
            (0.4..0.65).contains(&ratio),
            "wet stride ≈ half the dry one: {ratio}"
        );
        // Momentum survives: the velocity is not scaled, only the stride.
        assert!(p.vx > 2.0, "vx preserved in water: {}", p.vx);
    }

    /// `Player.cs:20059-20087`: no input decays by `runSlowdown` on the ground
    /// and half of it in the air (0.2 vs 0.1 per tick).
    #[test]
    fn air_friction_is_half_the_ground_friction() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.vx = 2.0;
        p.on_ground = false;
        p.vy = -3.0; // airborne
        p.update(&w, PlayerInput::default());
        let air = 2.0 - p.vx;
        let mut q = Player::spawn(&w);
        q.vx = 2.0;
        q.on_ground = true;
        q.vy = 0.0;
        q.update(&w, PlayerInput::default());
        let ground = 2.0 - q.vx;
        assert!(
            (air - ground * 0.5).abs() < 1e-4,
            "air {air} vs ground {ground}"
        );
    }

    /// The port used to hard-clamp vx to ±3 every tick — the vanilla has no
    /// clamp: a knockback-speed vx (4.5) decays by friction instead.
    #[test]
    fn overspeed_rides_out() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.vx = 4.5;
        p.update(&w, PlayerInput::default());
        assert!(
            p.vx > 4.0 && p.vx < 4.5,
            "no clamp, friction only: vx={}",
            p.vx
        );
    }

    /// `Player.cs:23755-23783`: `GetFloorTile` samples the center column
    /// first, then the neighbors — the flags come from the tile under the
    /// feet (one tick stale, so a settled player has live flags).
    #[test]
    fn floor_flags_sample_the_feet_row() {
        let w = floor_world(161);
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.on_ground, "must settle first");
        assert!(p.slippy, "ice floor flags the slippery set");
        let mut q = Player::spawn(&ground_world());
        for _ in 0..120 {
            q.update(&ground_world(), PlayerInput::default());
        }
        q.update(&ground_world(), PlayerInput::default());
        assert!(!q.slippy, "dirt is not slippery");
    }

    /// `Player.cs:25363`: `(fell - (25 + extraFall)) * 10`, `fell` truncated on
    /// both ends. Dropping from tile 25 lands at row 57 → fell = 32 → 70.
    #[test]
    fn fall_damage_matches_the_vanilla_formula() {
        let w = ground_world();
        let (lost, landed) = drop_and_settle(&w, 25);
        assert_eq!(landed, 57, "feet rest on row 57");
        assert_eq!(lost, (landed - 25 - 25) * 10, "fell = 57 - 25 = 32 tiles");
    }

    /// `Player.cs:25360`: `num11 > num10` is strict — a 25-tile fall hurts for
    /// nothing; the first damaging tile is 26.
    #[test]
    fn the_safe_fall_threshold_is_strict() {
        let w = ground_world();
        // From tile 32 the feet land at row 57 → fell 25 → no damage.
        let (lost, _) = drop_and_settle(&w, 32);
        assert_eq!(lost, 0, "25-tile fall is free");
    }

    /// `Player.cs:25361`: the normal branch forces `immune = false` BEFORE the
    /// Hurt — a fall lands through any iframes the contact damage granted.
    #[test]
    fn fall_damage_ignores_iframes() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.hurt_cd = 5.0; // mid-iframes
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert!(p.hp < 100, "fall must land through iframes");
    }

    /// The liquid reset (`Player.cs:25379`): while `wet` the fall origin rides
    /// the player, so a fall through water lands for nothing.
    #[test]
    fn water_forgives_the_fall() {
        let mut w = ground_world();
        for y in 50..58 {
            for x in 0..64 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 200;
            }
        }
        let (lost, _) = drop_and_settle(&w, 20);
        assert_eq!(lost, 0, "a fall through water is free");
    }

    /// `Player.cs:25338-25345`: a landing on the cloud set (tile 189 here)
    /// zeroes the fall distance.
    #[test]
    fn clouds_forgive_the_fall() {
        let mut w = ground_world();
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i].kind = 189;
        }
        let (lost, _) = drop_and_settle(&w, 25);
        assert_eq!(lost, 0, "a landing on clouds is free");
    }

    /// `Player.cs:25347-25359`: petrified replaces the whole formula — a fixed
    /// 2-tile grace at 20 damage per tile, ignoring extraFall.
    #[test]
    fn stoned_landing_uses_the_petrified_formula() {
        let mut w = ground_world();
        // Clouds out of the picture: the plain dirt surface again.
        let mut p = Player::spawn(&w);
        p.max_hp = 1000;
        p.hp = 1000;
        p.buffs.add_buff(156, 600); // Petrified
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        let landed = (p.y / TILE_PX) as i32;
        // (fell - 2) * 20 = (32 - 2) * 20 = 600, minus the flat petrify hit of
        // 20 taken the moment the buff appeared.
        assert_eq!(p.hp, 1000 - (landed - 25 - 2) * 20 - 20, "petrified hp");
    }

    /// `Player.cs:25370`: fall damage calls Hurt with hitDirection 0 — no
    /// knockback, the player stays planted where they landed.
    #[test]
    fn fall_landing_has_no_knockback() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        p.y = 25.0 * TILE_PX;
        p.fall_start = 25;
        for _ in 0..600 {
            p.update(&w, PlayerInput::default());
            if p.on_ground {
                break;
            }
        }
        assert!(p.on_ground, "must rest after landing");
        assert_eq!(p.vy, 0.0, "no upward knock");
    }

    #[test]
    fn idle_uses_the_rest_pose_row_zero() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        // `PlayerFrame`: standing still is leg/body frame row 0, never the jump
        // pose (row 5) that made the character look like it was about to leap.
        assert_eq!(p.leg_frame, 0, "idle legs must be row 0");
        assert_eq!(p.body_frame, 0, "idle body must be row 0");
    }

    #[test]
    fn airborne_uses_row_five() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        p.update(
            &w,
            PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        for _ in 0..8 {
            p.update(&w, PlayerInput::default());
        }
        assert!(p.vy != 0.0, "should be airborne, vy={}", p.vy);
        assert_eq!(p.leg_frame, 5, "airborne legs must be row 5");
        assert_eq!(p.body_frame, 5, "airborne body must be row 5");
    }

    #[test]
    fn walking_cycles_the_walk_rows() {
        let w = ground_world();
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..240 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
            assert!(
                (7..=19).contains(&p.leg_frame),
                "walk row {} outside 7..=19",
                p.leg_frame
            );
            assert_eq!(p.body_frame, p.leg_frame, "body must copy the leg row");
            seen.insert(p.leg_frame);
        }
        assert!(seen.len() >= 4, "walk cycle did not advance: {seen:?}");
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
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = w.index(19, 60);
        w.tiles[i].set_slope(1);
        w.spawn_x = 10;
        w.spawn_y = 58;
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut last = p.y + p.gfx_off_y;
        let mut max_step: f32 = 0.0;
        for _ in 0..180 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
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
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, PlayerInput::default());
        }
        let mut last = p.y + p.gfx_off_y;
        let mut max_step: f32 = 0.0;
        let mut stood_on_step = false;
        for _ in 0..240 {
            p.update(
                &w,
                PlayerInput {
                    right: true,
                    ..Default::default()
                },
            );
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
        p.regen.life_regen_time = 0.0;
        for _ in 0..1200 {
            p.update(&w, PlayerInput::default());
        }
        // The faithful engine at `statLifeMax2 = 100` is slow: the ramp scale is
        // `100/400·0.85 + 0.15 = 0.3625`, so tiers 1-2 round banker's-style to
        // 0 and the first points arrive from tick ~600 on (≈ +4 hp over 20 s
        // standing). The old placeholder granted 1 hp per half-second — the
        // `> 60` bar belonged to it.
        assert!(p.hp > 50, "regen only reached {} (sit still, 20 s)", p.hp);
        assert!(p.hp <= 56, "regen overshot the faithful pace: {}", p.hp);
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
