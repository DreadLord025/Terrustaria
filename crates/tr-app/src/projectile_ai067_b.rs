//! `AI_067_FreakingPirates`, slice B (S1-D-4b-12): the three RNG-light pets —
//! the Crimson Heart (500), the Dirtiest Block (1018) and the Boulder Pet
//! (1056) — on the sibling slice's skeleton (`projectile.rs`
//! `ai_freaking_pirates`, S1-D-4b-7). The C# arm is `Projectile.cs:49121-51201`
//! (dispatch `:30162`); every block below cites its lines.
//!
//! The pets set `flag13 = false` (`:49556-49560`), so the target scan
//! (`:49624-49632`), the pursuit (`:50103-50270`) and the bite/teleport states
//! 2/5 are unreachable — `ai[0]` only ever rides 0 (walk) and 1 (fly home,
//! `:49773`/`:50293`). The walk's frame machines (`:50768-51143`) have no pet
//! arm; the pets' frames come from the per-tick blocks (`:49462-49493`).
//!
//! **RNG proof (by construction).** The only arm-level draws on these three
//! types are the dust-velocity `RotatedByRandom` pairs:
//! * 500, every tick (`:49482`): `RotatedByRandom(0.7853981852531433)` — TWO
//!   `NextDouble` draws (`Utils.cs:1697-1700`, left operand first). The draw
//!   runs even when `NewDust` returns the 6000 sentinel (the velocity write
//!   targets `Main.dust[6000]`), so it is unconditional.
//! * 1018, state 1 only when `Main.LocalPlayer.miscCounter % 3 == 0`
//!   (`:49946-49952`): one pair (0.2617993950843811).
//! * 1018, the landing burst (`:50717-50727`): SIX pairs, once per landing
//!   (`localAI[2]` 0→1 on the ground, →0 airborne `:50729-50732`).
//! The `NewDust` internal rolls belong to the dust pool and are skipped (the
//! S1-D-4b-7 precedent); the arm-level pairs stay on the stream as two
//! `next_float` calls each — `UnifiedRandom.next_float` IS `next_double`
//! (`unified_random.rs:209-211`), so the stream advances identically; the
//! values are discarded with the dust pool, so the f32 cast is unobservable.
//! No other draw exists on these types: no scan, no bite, no burst, no blink.
//! **W4-d7 F3** landed the two deferred pets: the 653 Companion Cube (the
//! scream `Next(10)` + the darkness `Next(30,120)`/`Next(5)`/`Next(30)` chain
//! with the `Player.Hurt`, `:49494-49554`, `companion_cube_tick` below) and
//! the 1090 Rainbow Boulder Pet (the `NextVector2CircularEdge` shimmer +
//! `RotatedByRandom` steering, `:49744-49818`). Their draws feed observable
//! state, so each stays verbatim on the stream. The 1036 Cob Whip spider's
//! `Kill()` dust census (`:71068-71076`) rides
//! [`cob_whip_spider_kill_draws`]; the spider's AI is the sibling
//! `ai_freaking_pirates`' `flag7` arm.
//!
//! Deviations (the standing S1-D-4b-7 precedents, ANALYSIS "The pirate family,
//! first slice" + "slice B"): the dust spawns and the 500's
//! `Lighting.AddLight` (`:49464`) have no port layer; `flag10` (petting) folds
//! to false; `base.wet` folds to false; the blocked-move chain
//! (`:17904-19854`) has no 500/1018/1056 type arm, so a blocked move only
//! stops them. The 500's `correctSlopeCollision` is FALSE
//! (`Main.projPet[500]` is never set — `Main.cs:8728-8846` sets 492/499 but
//! not 500, `Projectile.cs:707`), unlike the sibling's kinds; the port's
//! `tile_collision` has no `correctSlopeCollision` parameter layer, so the
//! extra `SlopeCollision` passes the C# would run for the 500 fold into the
//! port's single slope-aware pass — documented in ANALYSIS.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::player::Player;
use crate::projectile::{
    BOULDER_PET, COMPANION_CUBE, CRIMSON_HEART, DIRTIEST_BLOCK, RAINBOW_BOULDER_PET, Projectile,
    angle_lerp, angle_towards, box_solid, csign, get_lerp_value, proj_frames, safe_normalize,
    solid_tile, wrap_angle,
};

/// The pets' idle home spot (`Projectile.cs:49441-49453`): the 500 stands
/// 15+width/2+40 px behind the owner's facing, the 1018 dead on the owner's
/// centre X, the 1056 15+width/2 px behind. Pure — the ladder's pet half.
pub(crate) fn pet_home_spot(kind: u16, player_center: Vec2, facing: i32) -> Vec2 {
    let mut vector = player_center;
    match kind {
        // `:49441-49445`
        CRIMSON_HEART => {
            vector.x -= (15.0 + crate::player::WIDTH * 0.5) * facing as f32;
            vector.x -= 40.0 * facing as f32;
        }
        // `:49446-49449` — `vector.X = player.Center.X` (a no-op on the
        // centre-initialised spot, kept for the citation). The 653 shares it
        // (`:49458-49461`, W4-d7 F3).
        DIRTIEST_BLOCK | COMPANION_CUBE => {
            vector.x = player_center.x;
        }
        // `:49450-49453` — the 1056; the 1090 shares it (`:49454-49457`,
        // W4-d7 F3).
        BOULDER_PET | RAINBOW_BOULDER_PET => {
            vector.x -= (15.0 + crate::player::WIDTH * 0.5) * facing as f32;
        }
        _ => {}
    }
    vector
}

/// `HandleMovement`'s effective `flag6` for the pets
/// (`Projectile.cs:17478-17503`): the 500 and the 1018 fall through only when
/// the owner's bottom is 4+ px below theirs (`:17488-17503`); the 1056 rides
/// the generic `projPet` rule (`:17480-17487`, `Main.projPet[1056] = true`,
/// `Main.cs:8846`) — the owner's bottom −12 above theirs. The arm's
/// `shouldFallThrough` write (`:49561`) is dead for the pets
/// (`decidesManualFallThrough` is false, `:17583-17586`), so this value rides
/// the slot's `should_fall_through` into `pirate_movement`.
pub(crate) fn pet_fall_through(kind: u16, player_bottom: f32, proj_bottom: f32) -> bool {
    match kind {
        // `:17488-17495` (500) / `:17496-17503` (1018, shared with the 653 —
        // W4-d7 F3).
        CRIMSON_HEART | DIRTIEST_BLOCK | COMPANION_CUBE => player_bottom > proj_bottom + 4.0,
        // `:17480-17487` — the projPet rule (1056/1090; `Main.projPet[1018]`
        // is also true but the `:17496` type arm overrides it afterwards).
        _ => player_bottom - 12.0 > proj_bottom,
    }
}

impl Projectile {
    /// `AI_067_FreakingPirates` (`Projectile.cs:49121-51201`) for the three
    /// slice-B pets (500/1018/1056). The sibling's `ai_freaking_pirates`
    /// keeps the 393/394/395/758 kinds byte-identical; the dispatch's slice-B
    /// arm routes only these three here. `npcs` is not a parameter: the pets
    /// never scan (`flag13 = false`, `:49556-49560`).
    pub(crate) fn ai_freaking_pirates_pets(
        &mut self,
        world: &World,
        mut player: Option<&mut Player>,
        rng: &mut dyn DropRng,
        cube_light: Option<(f32, f32)>,
    ) {
        // `:49133` — the gravity base.
        let num = 0.4f32;
        // `:49147-49155` — no active owner: deactivate (the C# writes
        // `active = false` directly, not `Kill()`).
        if player.is_none() {
            self.active = false;
            return;
        }
        // `:49156-49160` — the parameter ladder (`num3 = 500` default);
        // `flag10` (petting) folds to false (the port has no petting system,
        // the sibling precedent).
        let num4 = 300.0f32;
        // `:49328-49387` — the pets' keep-alive pins: `num3 = 200` for the
        // 500/1018/1056/1090, `num3 = 300` for the 653 (`:49342`, W4-d7 F3);
        // the buff flags ride `Player.pet_flags` (the `buff_effects_accessory`
        // granters, `Player.cs:11019-11032` etc.).
        let num3 = if self.kind == COMPANION_CUBE { 300.0f32 } else { 200.0f32 };
        {
            let p = player.as_deref_mut().unwrap();
            let held = p.pet_flags[self.kind as usize];
            if p.dead {
                p.pet_flags[self.kind as usize] = false;
            } else if held {
                self.time_left = 2;
            }
        }
        // The player scalars the pet machine reads (snapshot before the
        // per-tick match — the Companion Cube's block re-borrows `player`
        // mutably for the darkness bite's Hurt, so no immutable `p` borrow
        // may be live across it).
        let (p_misc_counter, p_vx, p_vy, player_center, player_bottom, p_facing) = {
            let pl = player.as_deref().unwrap();
            let (pcx, pcy) = pl.center();
            (pl.misc_counter, pl.vx, pl.vy, Vec2::new(pcx, pcy), pl.y + pl.height, pl.facing)
        };
        // `:49405` + `:49441-49453` — the idle home spot.
        let mut vector = pet_home_spot(self.kind, player_center, p_facing);
        // `:49462-49493` — the per-tick pet blocks, BEFORE the state machine.
        match self.kind {
            CRIMSON_HEART => {
                // `:49464` — the AddLight has no port layer (the module doc).
                // `:49465-49477` — the 4-frame pulse: frames 0/2 hold 12
                // ticks, the others 6.
                let num7 = if self.frame == 0 || self.frame == 2 { 12 } else { 6 };
                self.frame_counter += 1;
                if self.frame_counter >= num7 {
                    self.frame_counter = 0;
                    self.frame += 1;
                    if self.frame >= proj_frames(self.kind) as i32 {
                        self.frame = 0;
                    }
                }
                // `:49478`
                self.rot += self.vel.x / 20.0;
                // `:49479-49484` — the dust spawn is skipped; the
                // `RotatedByRandom(0.785…)` pair stays on the stream (TWO
                // draws, `Utils.cs:1697-1700`; the module doc's RNG proof).
                let _ = (rng.next_float(), rng.next_float());
            }
            // `:49486-49489`
            DIRTIEST_BLOCK => {
                self.rot += self.vel.x / 20.0;
            }
            // `:49490-49493` — the 1056/1090 share the /16 spin (W4-d7 F3).
            BOULDER_PET | RAINBOW_BOULDER_PET => {
                self.rot += self.vel.x / 16.0;
            }
            // `:49494-49555` — the Companion Cube (W4-d7 F3): the /20 spin,
            // the lava scream and the darkness bite. The scream cooldown is
            // a per-owner static (`_CompanionCubeScreamCooldown`, `:49500`);
            // the port folds it to the slot's `local_ai[2]` (one cube per
            // owner, the pet-flag convention — recorded in ANALYSIS). The
            // darkness read needs the light map (the pool's `cube_light`
            // snapshot, the `set_disco` precedent); `None` folds the bite
            // inert (the pool-only tests).
            COMPANION_CUBE => {
                self.rot += self.vel.x / 20.0;
                self.companion_cube_tick(world, player.as_deref_mut(), rng, cube_light);
            }
            _ => {}
        }
        // `:49556-49562` — `flag13 = false` (no scan), the fall-through
        // decision (the effective `HandleMovement` rule, the module doc) and
        // the friendly reset. `num9 = 0`, `num10 = 15`, `attackTarget = -1`
        // (`:49563-49565`) — no pet arm touches them.
        self.should_fall_through = pet_fall_through(self.kind, player_bottom, self.pos.y + self.h);
        self.friendly = false;
        // `:49624-49632` — `flag16 & flag13` is false: no target scan. The
        // states 2/5 (`:49954-50098`) and the pursuit (`:50103-50270`) are
        // unreachable for the pets (nothing arms them).
        // `:49742-49953` — state 1: fly back to the owner.
        if self.ai[0] == 1.0 {
            // `:49744-49752` — the 1090's shimmer particle (W4-d7 F3): the
            // miscCounter%2 gate, the NextVector2CircularEdge(w/2,h/2) draw
            // (ONE NextFloat, Utils.cs:1261-1263/1271-1273) stays on the
            // stream; the particle body is the render skip.
            if self.kind == RAINBOW_BOULDER_PET && p_misc_counter % 2 == 0 {
                let _ = rng.next_float();
            }
            self.tile_collide = false;
            let num17 = 0.2f32;
            let mut num18 = 10.0f32;
            let num19 = 200.0f32;
            // `:49757-49760` — the 1090's speed floor is 18 (W4-d7 F3).
            if self.kind == RAINBOW_BOULDER_PET {
                num18 = 18.0;
            }
            let pv_speed = p_vx.abs() + p_vy.abs();
            if num18 < pv_speed {
                num18 = pv_speed;
            }
            let center = self.pos + Vec2::new(self.w * 0.5, self.h * 0.5);
            let spinninpoint = player_center - center;
            let num20 = spinninpoint.length();
            if num20 > 2000.0 {
                // `:49767-49770` — the teleport home.
                self.pos = player_center - Vec2::new(self.w, self.h) * 0.5;
            }
            // `:49771-49779` — arrived below the owner's feet: land.
            if num20 < num19
                && p_vy == 0.0
                && self.pos.y + self.h <= player_bottom
                && !box_solid(world, self.pos, self.w, self.h)
            {
                self.ai[0] = 0.0;
                self.net_update = true; // `:49774`
                if self.vel.y < -6.0 {
                    self.vel.y = -6.0;
                }
            }
            // `:49780-49785` — the Cattiva mount snap is deferred.
            let mut num21 = 60.0f32;
            // `:49787-49790` — the 1090's close radius is 350 (W4-d7 F3).
            if self.kind == RAINBOW_BOULDER_PET {
                num21 = 350.0;
            }
            if num20 < num21 {
                // `:49791-49802` — close: bleed the speed; the 1090 also
                // renormalizes to num18 when slower (W4-d7 F3).
                self.local_ai[0] = 0.0;
                if self.kind == RAINBOW_BOULDER_PET && self.vel.length() < num18 {
                    if self.vel == Vec2::ZERO {
                        self.vel = Vec2::new(0.0, -1.0);
                    }
                    self.vel = safe_normalize(self.vel) * num18;
                }
            } else {
                // `:49803-49855` — far: steer at the owner. The 1090's
                // `RotatedByRandom(0.5236)` arm (`:49807-49819`, W4-d7 F3)
                // replaces the per-axis accel: TWO NextDouble draws
                // (Utils.cs:1697-1700), the bounce particle skipped.
                if self.kind == RAINBOW_BOULDER_PET {
                    let aim = safe_normalize(spinninpoint) * num18;
                    let a = rng.next_double() * 0.5235987901687622
                        - rng.next_double() * 0.5235987901687622;
                    self.vel = crate::bow_launch::rotated_by(aim, a);
                    if self.local_ai[0] == 0.0 {
                        self.local_ai[0] = 1.0;
                    }
                } else {
                let aim = safe_normalize(spinninpoint) * num18;
                if self.vel.x < aim.x {
                    self.vel.x += num17;
                    if self.vel.x < 0.0 {
                        self.vel.x += num17 * 1.5;
                    }
                }
                if self.vel.x > aim.x {
                    self.vel.x -= num17;
                    if self.vel.x > 0.0 {
                        self.vel.x -= num17 * 1.5;
                    }
                }
                if self.vel.y < aim.y {
                    self.vel.y += num17;
                    if self.vel.y < 0.0 {
                        self.vel.y += num17 * 1.5;
                    }
                }
                if self.vel.y > aim.y {
                    self.vel.y -= num17;
                    if self.vel.y > 0.0 {
                        self.vel.y -= num17 * 1.5;
                    }
                }
                }
            }
            // `:49856-49859` — the facing.
            if self.vel.x != 0.0 {
                self.sprite_direction = if self.vel.x > 0.0 { 1 } else { -1 };
            }
            // `:49860-49941` — no pet arm (the Flinx/pirate/Cattiva/frog/
            // tiger/Clay-Pot flight frames belong to the deferred kinds).
            // `:49942-49945` — the boulder's flight spin (the 1056/1090 share
            // it, W4-d7 F3).
            if self.kind == BOULDER_PET || self.kind == RAINBOW_BOULDER_PET {
                self.rot += 0.5 * self.sprite_direction as f32;
            }
            // `:49946-49952` — the Dirtiest Block's flight dust: the
            // `miscCounter % 3` gate, the spawn skipped, the
            // `RotatedByRandom(0.2617…)` pair on the stream.
            if self.kind == DIRTIEST_BLOCK && p_misc_counter % 3 == 0 {
                let _ = (rng.next_float(), rng.next_float());
            }
        }
        // `:50271-50300` — idle and far from the owner: fly back (`num3` is
        // the pets' 200; `flag7`/`flag8` arms don't apply).
        if self.ai[0] == 0.0 {
            let my_center = self.pos + Vec2::new(self.w * 0.5, self.h * 0.5);
            let vector7 = player_center - my_center;
            let mut flag18 = false;
            if vector7.length() > 2000.0 {
                self.pos = player_center - Vec2::new(self.w, self.h) * 0.5;
            } else if vector7.length() > num3 || vector7.y.abs() > num4 {
                flag18 = true;
                if self.vel.y > 0.0 && vector7.y < 0.0 {
                    self.vel.y = 0.0;
                }
                if self.vel.y < 0.0 && vector7.y > 0.0 {
                    self.vel.y = 0.0;
                }
            }
            if flag18 {
                self.ai[0] = 1.0;
                self.net_update = true; // `:50294`
            }
        }
        // `:50301-51149` — state 0: the ground walk (state 3 is the deferred
        // miners' — unreachable here).
        if self.ai[0] == 0.0 || self.ai[0] == 3.0 {
            // `:50303-50363` — the home-spot resolution (`attackTarget < 0`
            // always for the pets; the `:50309-50326` state-3 jump is
            // unreachable).
            let my_center = self.pos + Vec2::new(self.w * 0.5, self.h * 0.5);
            // `:50305-50308` — the home spot on the wrong side of the owner
            // snaps to the owner.
            if my_center.distance(player_center) > 60.0
                && my_center.distance(vector) > 60.0
                && csign(vector.x - player_center.x) != csign(my_center.x - player_center.x)
            {
                vector = player_center;
            }
            // `:50327-50336` — drop the home spot to the ground (20 rows).
            let size = Vec2::new(self.w, self.h);
            let mut r_top = vector - size * 0.5;
            for _ in 0..20 {
                if box_solid(world, r_top, self.w, self.h) {
                    break;
                }
                r_top.y += 16.0;
                vector.y += 16.0;
            }
            // `:50337-50338` — clamp the spot through the owner's tiles.
            let tc = crate::collision::tile_collision(
                world,
                [player_center.x - size.x * 0.5, player_center.y - size.y * 0.5],
                [vector.x - player_center.x, vector.y - player_center.y],
                self.w,
                self.h,
                false,
                false,
            );
            vector = player_center + Vec2::new(tc.vel[0], tc.vel[1]);
            // `:50339-50346` — already close: hold position.
            if my_center.distance(vector) < 32.0 && self.ai[0] != 3.0 {
                let num43 = player_center.distance(vector);
                if player_center.distance(my_center) < num43 {
                    vector = my_center;
                }
            }
            // `:50347-50363` — too far: walk the owner→spot line in 0.05
            // steps and stop at the first solid box.
            let vector9 = player_center - vector;
            if vector9.length() > num3 || vector9.y.abs() > num4 {
                let r2_top = player_center - size * 0.5;
                let vector10 = vector - player_center;
                let mut vector11 = r2_top;
                let mut num44 = 0.0f32;
                while num44 < 1.0 {
                    let vector12 = r2_top + vector10 * num44;
                    if box_solid(world, vector12, self.w, self.h) {
                        break;
                    }
                    vector11 = vector12;
                    num44 += 0.05;
                }
                vector = vector11 + size * 0.5;
            }
            self.tile_collide = true;
            // `:50366-50422` — the walk parameter ladder: no pet arm raises
            // it (the pets never attack), only the owner-speed raise.
            let mut num45 = 0.5f32;
            let num46 = 4.0f32;
            let mut num47 = 4.0f32;
            let num48 = 0.1f32;
            let pv_speed = p_vx.abs() + p_vy.abs();
            if num47 < pv_speed {
                num47 = pv_speed;
                num45 = 0.7;
            }
            // `:50423-50434` — the 1018/653 cap the walk speed at the owner's
            // actual speed (W4-d7 F3: the 653 shares the 1018 arm).
            if self.kind == DIRTIEST_BLOCK || self.kind == COMPANION_CUBE {
                let mut num49 = Vec2::new(p_vx, p_vy).length();
                if num49 < 0.1 {
                    num49 = 0.0;
                }
                if num49 != 0.0 && num49 < num47 {
                    num47 = num49;
                }
            }
            // `:50435-50443` — the horizontal steer (`flag20 = flag10` is
            // false).
            let mut num50 = 0i32;
            let mut flag19 = false;
            let my_center = self.pos + Vec2::new(self.w * 0.5, self.h * 0.5);
            let num51 = vector.x - my_center.x;
            let vector13 = vector - my_center;
            if (self.kind == BOULDER_PET || self.kind == RAINBOW_BOULDER_PET) && num51.abs() < 50.0 {
                // `:50444-50464` — the boulder's near-home settle (the
                // 1056/1090 share it, W4-d7 F3): the owner-speed cap, the
                // slow-spin angle lerp and the 0.9 bleed with the 0.1 snap.
                let mut num52 = Vec2::new(p_vx, p_vy).length();
                if num52 < 0.1 {
                    num52 = 0.0;
                }
                if num52 != 0.0 && num52 < num47 {
                    num47 = num52;
                }
                if self.vel.length() <= 0.3 {
                    self.rot = angle_lerp(wrap_angle(self.rot), 0.0, 0.1);
                }
                self.vel.x *= 0.9;
                if self.vel.x.abs() < 0.1 {
                    self.vel.x = 0.0;
                }
            } else if self.kind == DIRTIEST_BLOCK && num51.abs() < 50.0 {
                // `:50465-50473` — the block's near-home settle: the 0.2
                // angle-towards and the same bleed/snap.
                self.rot = angle_towards(self.rot, 0.0, 0.2);
                self.vel.x *= 0.9;
                if self.vel.x.abs() < 0.1 {
                    self.vel.x = 0.0;
                }
            } else if self.kind == COMPANION_CUBE && num51.abs() < 150.0 {
                // `:50474-50482` — the cube's near-home settle (W4-d7 F3):
                // the 150-px band, the 0.2 angle-towards, the same bleed/snap.
                self.rot = angle_towards(self.rot, 0.0, 0.2);
                self.vel.x *= 0.9;
                if self.vel.x.abs() < 0.1 {
                    self.vel.x = 0.0;
                }
            } else if num51.abs() > 5.0 {
                // `:50483-50508` — the steer ladder.
                if num51 < 0.0 {
                    num50 = -1;
                    if self.vel.x > 0.0 - num46 {
                        self.vel.x -= num45;
                    } else {
                        self.vel.x -= num48;
                    }
                } else {
                    num50 = 1;
                    if self.vel.x < num46 {
                        self.vel.x += num45;
                    } else {
                        self.vel.x += num48;
                    }
                }
                // `:50509-50557` — the jump intent: the 1018/653 never
                // auto-jump (`:50526-50533`, W4-d7 F3), the boulder only
                // above 5 speed (`:50534-50537`); the 500 keeps the stock true.
                let mut flag21 = true;
                if self.kind == DIRTIEST_BLOCK || self.kind == COMPANION_CUBE {
                    flag21 = false;
                }
                if (self.kind == BOULDER_PET || self.kind == RAINBOW_BOULDER_PET)
                    && self.vel.length() <= 5.0
                {
                    flag21 = false;
                }
                if flag21 {
                    flag19 = true;
                }
            } else {
                // `:50559-50566` — the dead band.
                self.vel.x *= 0.9;
                if self.vel.x.abs() < num45 * 2.0 {
                    self.vel.x = 0.0;
                }
            }
            // `:50567-50584` — the wall probe ahead.
            let flag22 =
                vector13.x.abs() >= 64.0 || (vector13.y <= -48.0 && vector13.x.abs() >= 8.0);
            if (num50 != 0) & flag22 {
                let mut num53 = (self.pos.x + self.w * 0.5) as i32 / 16;
                let num54 = self.pos.y as i32 / 16;
                num53 = num53 + num50 + self.vel.x as i32;
                for num55 in num54..num54 + self.h as i32 / 16 + 1 {
                    if solid_tile(world, num53, num55) {
                        flag19 = true;
                    }
                }
            }
            // `:50585-50600` — the pets' motion jump intents: the 500 jumps
            // whenever it moves sideways, the 1018/653 above 3 (W4-d7 F3),
            // the boulder above 5 total speed.
            if self.kind == CRIMSON_HEART && self.vel.x != 0.0 {
                flag19 = true;
            }
            if (self.kind == DIRTIEST_BLOCK || self.kind == COMPANION_CUBE) && self.vel.x.abs() > 3.0 {
                flag19 = true;
            }
            if (self.kind == BOULDER_PET || self.kind == RAINBOW_BOULDER_PET) && self.vel.length() > 5.0 {
                flag19 = true;
            }
            // `:50605` — the step-up.
            let mut pos = [self.pos.x, self.pos.y];
            let mut vel = [self.vel.x, self.vel.y];
            crate::collision::step_up(
                world,
                &mut pos,
                &mut vel,
                self.w,
                self.h,
                &mut self.step_speed,
                &mut self.gfx_off_y,
                false,
            );
            self.pos = Vec2::new(pos[0], pos[1]);
            self.vel = Vec2::new(vel[0], vel[1]);
            // `:50606-50608` — the fall-through lift (`flag23` is the
            // deferred 1022's; `num57` stays 1).
            let num56 =
                get_lerp_value(0.0, 100.0, vector13.y) * get_lerp_value(-2.0, -6.0, self.vel.y);
            if self.vel.y == 0.0 {
                if flag19 {
                    // `:50617-50663` — the three-column ground probe and the
                    // jump ladder (the last column's write wins).
                    for num58 in 0..3 {
                        let mut num59 = (self.pos.x + self.w * 0.5) as i32 / 16;
                        if num58 == 0 {
                            num59 = self.pos.x as i32 / 16;
                        }
                        if num58 == 2 {
                            num59 = (self.pos.x + self.w) as i32 / 16;
                        }
                        let num60 = (self.pos.y + self.h) as i32 / 16;
                        let ground = world.tile(num59, num60);
                        let stands = solid_tile(world, num59, num60)
                            || ground.is_some_and(|t| {
                                t.half_brick()
                                    || t.slope() > 0
                                    || (tr_core::tiledata::SET_PLATFORMS.has(t.kind)
                                        && t.active()
                                        && !t.inactive())
                            });
                        if !stands {
                            continue;
                        }
                        // `:50633-50662` — the try/catch: the first two
                        // probes' null tiles throw into the catch's -9.1;
                        // the later `SolidTile` calls read null as solid
                        // (`WorldGen.cs:70115-70117`).
                        let num59 =
                            (self.pos.x + self.w * 0.5) as i32 / 16 + num50 + self.vel.x as i32;
                        let num60 = (self.pos.y + self.h * 0.5) as i32 / 16;
                        if world.tile(num59, num60 - 1).is_none()
                            || world.tile(num59, num60 - 2).is_none()
                        {
                            self.vel.y = -9.1;
                        } else if !solid_tile(world, num59, num60 - 1)
                            && !solid_tile(world, num59, num60 - 2)
                        {
                            self.vel.y = -5.1;
                        } else if !solid_tile(world, num59, num60 - 2) {
                            self.vel.y = -7.1;
                        } else if solid_tile(world, num59, num60 - 5) {
                            self.vel.y = -11.1;
                        } else if solid_tile(world, num59, num60 - 4) {
                            self.vel.y = -10.1;
                        } else {
                            self.vel.y = -9.1;
                        }
                    }
                    // `:50664-50711` — the height ladder toward the aim.
                    let my_center = self.pos + Vec2::new(self.w * 0.5, self.h * 0.5);
                    if vector.y - my_center.y < -48.0 {
                        let num61 = (vector.y - my_center.y) * -1.0;
                        self.vel.y = if num61 < 60.0 {
                            -6.0
                        } else if num61 < 80.0 {
                            -7.0
                        } else if num61 < 100.0 {
                            -8.0
                        } else if num61 < 120.0 {
                            -9.0
                        } else if num61 < 140.0 {
                            -10.0
                        } else if num61 < 160.0 {
                            -11.0
                        } else if num61 < 190.0 {
                            -12.0
                        } else if num61 < 210.0 {
                            -13.0
                        } else if num61 < 270.0 {
                            -14.0
                        } else if num61 < 310.0 {
                            -15.0
                        } else {
                            -16.0
                        };
                    }
                    // `:50712-50715` — the wet double-jump folds out (no
                    // wet flag on the slot).
                }
                // `:50717-50727` — the Dirtiest Block's landing burst: once
                // per landing (`localAI[2]` 0→1), six dusts — the spawns
                // skipped, the six `RotatedByRandom(0.2617…)` pairs (TWELVE
                // draws) stay on the stream.
                if self.kind == DIRTIEST_BLOCK && self.local_ai[2] == 0.0 {
                    self.local_ai[2] = 1.0;
                    for _ in 0..6 {
                        let _ = (rng.next_float(), rng.next_float());
                    }
                }
            } else if self.kind == DIRTIEST_BLOCK {
                // `:50729-50732` — airborne: re-arm the landing burst.
                self.local_ai[2] = 0.0;
            }
            // `:50733-50743` — the speed cap.
            if self.vel.x > num47 {
                self.vel.x = num47;
            }
            if self.vel.x < 0.0 - num47 {
                self.vel.x = 0.0 - num47;
            }
            // `:50744-50767` — the facing (`ai[0] != 2` always here).
            if self.vel.x < 0.0 {
                self.direction = -1;
            }
            if self.vel.x > 0.0 {
                self.direction = 1;
            }
            if self.vel.x == 0.0 {
                self.direction = if player_center.x > my_center.x { 1 } else { -1 };
            }
            if self.vel.x > num45 && num50 == 1 {
                self.direction = 1;
            }
            if self.vel.x < 0.0 - num45 && num50 == -1 {
                self.direction = -1;
            }
            self.sprite_direction = self.direction;
            // `:50768-51143` — no pet frame machine (the whip/Mushroom
            // Boi/Flinx/pirate/Cattiva/Clay-Pot/frog/tiger blocks belong to
            // the deferred kinds); the pets' frames ride the per-tick blocks.
            // `:51144-51148` — the gravity + the fall-through lift.
            self.vel.y += num + num56;
            if self.vel.y > 10.0 {
                self.vel.y = 10.0;
            }
        }
        // `:51150-51171` — the pirates' idle burst doesn't apply to the pets.
        // `:51172-51200` — the 1090's frame cycle + localAI[2] fade (W4-d7 F3).
        if self.kind == RAINBOW_BOULDER_PET {
            self.frame_counter += 1;
            if self.frame_counter >= 2 {
                self.frame_counter = 0;
                self.frame += 1;
                if self.frame >= 100 {
                    self.frame = 0;
                }
            }
            let num70 = 0.025f32;
            if self.ai[0] == 1.0 {
                self.local_ai[2] += num70;
                if self.local_ai[2] >= 1.0 {
                    self.local_ai[2] = 1.0;
                }
            } else {
                self.local_ai[2] -= num70;
                if self.local_ai[2] <= 0.0 {
                    self.local_ai[2] = 0.0;
                }
            }
        }
    }

    /// `AI_067`'s Companion Cube per-tick block (`Projectile.cs:49494-49555`,
    /// W4-d7 F3): the lava scream (`localAI[0]` ramps in lava, kills at 20,
    /// the owner-gated scream cue on a 1/10 roll) and the darkness bite
    /// (`localAI[1]` ramps in the dark, the 1/5 fake-scream vs the 3-damage
    /// bite). The scream cooldown is the per-owner static
    /// `_CompanionCubeScreamCooldown` (`:49500`); the port folds it to the
    /// slot's `local_ai[2]` (one cube per owner, the pet-flag convention —
    /// ANALYSIS). The darkness read rides the pool's `cube_light` snapshot
    /// (the `set_disco` precedent); the caller passes it. `flag12` (the
    /// owner-slot gate) folds true (the pool simulates local-owned slots).
    fn companion_cube_tick(
        &mut self,
        world: &World,
        player: Option<&mut Player>,
        rng: &mut dyn DropRng,
        cube_light: Option<(f32, f32)>,
    ) {
        // `:49500-49504` — the scream cooldown decays (the slot fold).
        self.local_ai[2] -= 1.0;
        if self.local_ai[2] < 0.0 {
            self.local_ai[2] = 0.0;
        }
        // `:49506-49515` — the lava ramp: the centre tile's lava raises
        // localAI[0], else it falls; clamp [0, 20].
        let cx = (self.pos.x + self.w * 0.5) / tr_core::TILE_PX;
        let cy = (self.pos.y + self.h * 0.5) / tr_core::TILE_PX;
        let in_lava = world
            .tile(cx as i32, cy as i32)
            .filter(|t| t.liquid > 0)
            .is_some_and(|t| t.liquid_kind() == tr_core::tile::LiquidKind::Lava);
        if in_lava {
            self.local_ai[0] += 1.0;
        } else {
            self.local_ai[0] -= 1.0;
        }
        self.local_ai[0] = self.local_ai[0].clamp(0.0, 20.0);
        // `:49516-49524` — the scream + kill at 20. The C# does NOT return
        // here — the darkness block (`:49525`) still runs on the kill tick.
        if self.local_ai[0] >= 20.0 {
            if self.local_ai[2] == 0.0 {
                self.local_ai[2] = 3600.0;
                // `:49521` — Next(10)==0 → NPCDeath61 (4,61) else NPCDeath59
                // (4,59). The draw stays on the stream.
                let style = if rng.next_range(0, 10) == 0 { 61 } else { 59 };
                self.wants_sound = Some((4, style, self.pos.x, self.pos.y, 0.0, 1.0));
            }
            self.kill_time_left = self.time_left;
            self.active = false;
            self.time_left = 0;
        }
        // `:49525-49554` — the darkness bite (local owner only). The light
        // snapshot is the two GetColor lengths; None folds it inert.
        let Some((cube_len, player_len)) = cube_light else {
            return;
        };
        let Some(p) = player else {
            return;
        };
        if cube_len < 0.15 && player_len < 0.15 {
            self.local_ai[1] += 1.0;
        } else if self.local_ai[1] > 0.0 {
            self.local_ai[1] -= 1.0;
        }
        self.local_ai[1] = self.local_ai[1].clamp(-3600.0, 120.0);
        // `:49538` — the Next(30,120) threshold draw runs FIRST (the &&
        // short-circuits on the player gates AFTER it).
        let threshold = rng.next_range(30, 120) as f32;
        if self.local_ai[1] > threshold && !p.immune && p.vx == 0.0 && p.vy == 0.0 {
            if rng.next_range(0, 5) == 0 {
                // `:49542-49543` — the fake scream: Item16, localAI[1] = -600.
                self.wants_sound = Some((2, 16, self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5, 0.0, 1.0));
                self.local_ai[1] = -600.0;
            } else {
                // `:49547-49551` — the bite: Item1, Hurt(3), the i-frame
                // clear, localAI[1] = -300 + Next(30)*-10. The C# passes
                // hitDirection 0 — `Hurt`'s knockback gate (`Player.cs:38519`
                // `hitDirection != 0`) takes NO pop; the port's `hurt_inner`
                // with knock_back false (the fall lane's precedent).
                self.wants_sound = Some((2, 1, self.pos.x + self.w * 0.5, self.pos.y + self.h * 0.5, 0.0, 1.0));
                let cx = self.pos.x + self.w * 0.5;
                p.hurt_inner(3, cx, false, rng, true, false, None, 0);
                p.immune = false;
                p.immune_time = 0;
                // The port's collapsed cooldown rides `hurt_cd` too — clear
                // it with the i-frames or the next bite is swallowed (the
                // C# has no such second gate).
                p.hurt_cd = 0.0;
                self.local_ai[1] = -300.0 + rng.next_range(0, 30) as f32 * -10.0;
            }
        }
    }
}

/// `Kill()`'s 1036 dust census (`Projectile.cs:71068-71076`, W4-d7 F3):
/// 3× dust 5 (`NewDustPerfect`), each with ONE `NextVector2Circular(4,4)`
/// position roll — TWO `NextFloat` draws (`Utils.cs:1266-1268`: the
/// `NextVector2Unit` angle + the radius scale). The dust BODIES are the
/// documented skip (no draw layer); the six float draws stay on the stream.
/// The wiring caller gates the kind.
pub(crate) fn cob_whip_spider_kill_draws(rng: &mut dyn DropRng) {
    for _ in 0..3 {
        let _ = rng.next_float(); // the unit angle (`:1263`)
        let _ = rng.next_float(); // the radius scale (`:1268`)
    }
}

#[cfg(test)]
mod s1_d_4b_12_tests {
    use super::*;
    use crate::projectile::proj_data;

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn player(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.facing = 1;
        p
    }

    fn pet(kind: u16, w: f32, h: f32, pos: Vec2) -> Projectile {
        // Field-by-field on `Default` — the struct has private bookkeeping
        // fields (E0451 outside projectile.rs).
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.w = w;
        p.h = h;
        p.pos = pos;
        p
    }

    /// A draw-recording `Main.rand` stand-in (the sibling's `CensusRng`
    /// shape): the census proves count AND order. The pets draw no integer
    /// ranges at all — any `next_range` call is a port bug.
    struct CensusRng {
        ranges: Vec<(i32, i32)>,
        floats: usize,
    }

    impl DropRng for CensusRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.ranges.push((min, max));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.floats += 1;
            0.0
        }
    }

    /// The three rows' `SetDefaults` slice (`Projectile.cs:5413-5424`,
    /// `:9637-9647`, `:9987-9997`) + the frame sheets (`Main.cs:8484`; the
    /// 1018/1056 default 1, `Main.cs:8441`).
    #[test]
    fn s1_d_4b_12_the_three_rows_carry_the_csharp_set_defaults() {
        for (kind, w, h) in [(CRIMSON_HEART, 20.0, 20.0), (DIRTIEST_BLOCK, 16.0, 16.0), (BOULDER_PET, 32.0, 32.0)] {
            let d = proj_data(kind).unwrap();
            assert_eq!((d.w, d.h), (w, h), "kind {kind}");
            assert_eq!(d.ai_style, 67, "kind {kind}");
            assert_eq!(d.penetrate, -1, "kind {kind}");
            assert_eq!(d.time_left, 18000, "the `timeLeft *= 5`");
            assert!(d.tile_collide, "the stock true — the arms set neither");
            assert!(d.ignore_water, "kind {kind} sets `ignoreWater`");
            assert!(d.net_important, "kind {kind} sets `netImportant`");
            assert!(!d.minion, "the pets are not minions");
            assert!(!d.decides_manual_fall_through, "the pets keep the stock false");
            assert!(!d.uses_local_immunity, "kind {kind}");
            assert!(!d.uses_id_static_immunity, "kind {kind}");
        }
        assert_eq!(proj_data(CRIMSON_HEART).unwrap().scale, 0.8, "`:5423`");
        assert_eq!(proj_frames(CRIMSON_HEART), 4, "`Main.cs:8484`");
        assert_eq!(proj_frames(DIRTIEST_BLOCK), 1, "the init default");
        assert_eq!(proj_frames(BOULDER_PET), 1, "the init default");
    }

    /// `:49441-49453` — the home-spot ladder's pet half.
    #[test]
    fn s1_d_4b_12_the_home_spots_match_the_ladder() {
        let c = Vec2::new(1000.0, 500.0);
        // 15 + WIDTH/2 = 25; the 500 adds 40 more.
        assert_eq!(pet_home_spot(CRIMSON_HEART, c, 1), Vec2::new(935.0, 500.0));
        assert_eq!(pet_home_spot(CRIMSON_HEART, c, -1), Vec2::new(1065.0, 500.0));
        assert_eq!(pet_home_spot(DIRTIEST_BLOCK, c, 1), c);
        assert_eq!(pet_home_spot(BOULDER_PET, c, 1), Vec2::new(975.0, 500.0));
        assert_eq!(pet_home_spot(BOULDER_PET, c, -1), Vec2::new(1025.0, 500.0));
    }

    /// `HandleMovement`'s effective `flag6` (`:17478-17503`): the 500/1018
    /// +4 rule vs the 1056 projPet −12 rule.
    #[test]
    fn s1_d_4b_12_the_fall_through_rules_split_the_pets() {
        // The owner's bottom 10 px below the pet's: the +4 rule falls, the
        // −12 rule does not.
        for kind in [CRIMSON_HEART, DIRTIEST_BLOCK] {
            assert!(pet_fall_through(kind, 510.0, 500.0), "kind {kind}: +4");
            assert!(!pet_fall_through(kind, 504.0, 500.0), "kind {kind}: boundary");
        }
        assert!(!pet_fall_through(BOULDER_PET, 510.0, 500.0), "the −12 rule");
        assert!(pet_fall_through(BOULDER_PET, 513.0, 500.0), "the −12 rule");
    }

    /// `:49328-49375` — the keep-alive pins: the buff flag pins
    /// `timeLeft = 2`, death clears the flag (and does not pin).
    #[test]
    fn s1_d_4b_12_the_keep_alive_pins_ride_pet_flags() {
        let w = world();
        for kind in [CRIMSON_HEART, DIRTIEST_BLOCK, BOULDER_PET] {
            let mut p = player(&w);
            p.pet_flags[kind as usize] = true;
            let mut proj = pet(kind, 16.0, 16.0, Vec2::new(100.0, 100.0));
            proj.time_left = 500;
            let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
            proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
            assert_eq!(proj.time_left, 2, "kind {kind}: the pin");
            p.dead = true;
            proj.time_left = 500;
            proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
            assert!(!p.pet_flags[kind as usize], "kind {kind}: death clears");
            assert_eq!(proj.time_left, 500, "kind {kind}: no pin when dead");
        }
    }

    /// RNG census, the 500 (`:49462-49485`): exactly TWO float draws per
    /// tick (the dust's `RotatedByRandom` pair), no integer draws, and the
    /// 6/12-tick frame pulse wraps at `projFrames = 4`.
    #[test]
    fn s1_d_4b_12_the_crimson_heart_draws_one_dust_pair_per_tick() {
        let w = world();
        let mut p = player(&w);
        let (cx, cy) = p.center();
        // On the home spot (cx − 65): the walk's dead band keeps vel.x at 0.
        let mut proj = pet(CRIMSON_HEART, 20.0, 20.0, Vec2::new(cx - 75.0, cy - 10.0));
        let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
        for _ in 0..5 {
            proj.vel = Vec2::ZERO;
            proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        }
        assert_eq!(census.floats, 10, "two draws per tick, five ticks");
        assert!(census.ranges.is_empty(), "the pets draw no integer ranges");
        // The frame pulse: frame 0 holds 12 ticks — after 5 ticks the frame
        // is still 0 and the counter reads 5.
        assert_eq!(proj.frame, 0);
        assert_eq!(proj.frame_counter, 5);
        // The rotation accumulates vel.x / 20 (`:49478`).
        proj.vel.x = 2.0;
        let rot = proj.rot;
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        assert!((proj.rot - rot - 0.1).abs() < 1e-6, "`rotation += velocity.X / 20`");
    }

    /// RNG census, the 1018 in state 1 (`:49946-49952`): the dust pair draws
    /// ONLY when `miscCounter % 3 == 0`.
    #[test]
    fn s1_d_4b_12_the_dirtiest_block_flight_dust_rides_misc_counter() {
        let w = world();
        let mut p = player(&w);
        let (cx, cy) = p.center();
        let mut proj = pet(DIRTIEST_BLOCK, 16.0, 16.0, Vec2::new(cx + 300.0, cy));
        proj.ai[0] = 1.0;
        let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
        for tick in 0..6u32 {
            p.misc_counter = tick;
            proj.ai[0] = 1.0;
            let before = census.floats;
            proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
            let drawn = census.floats - before;
            assert_eq!(drawn, if tick % 3 == 0 { 2 } else { 0 }, "tick {tick}");
        }
        assert!(census.ranges.is_empty());
    }

    /// RNG census + state, the 1018's landing burst (`:50717-50732`): the
    /// first grounded tick draws TWELVE floats and sets `localAI[2] = 1`;
    /// the next grounded tick is silent; airborne re-arms.
    #[test]
    fn s1_d_4b_12_the_dirtiest_block_landing_burst_draws_six_pairs_once() {
        let w = world();
        let mut p = player(&w);
        let (cx, cy) = p.center();
        // On the home spot (the owner's centre X): the near-home settle
        // keeps vel.x at 0, so no landing re-trigger from motion.
        let mut proj = pet(DIRTIEST_BLOCK, 16.0, 16.0, Vec2::new(cx - 8.0, cy - 8.0));
        let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
        proj.vel = Vec2::ZERO;
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        assert_eq!(census.floats, 12, "six RotatedByRandom pairs");
        assert_eq!(proj.local_ai[2], 1.0, "the burst latch");
        proj.vel = Vec2::ZERO;
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        assert_eq!(census.floats, 12, "the latch holds — no second burst");
        // Airborne: the latch re-arms (`:50729-50732`), no draws.
        proj.vel.y = 2.0;
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        assert_eq!(proj.local_ai[2], 0.0, "airborne re-arms");
        assert_eq!(census.floats, 12);
        assert!(census.ranges.is_empty());
    }

    /// RNG census, the 1056: ZERO draws on any tick, and the near-home
    /// settle bleeds vel.x by 0.9 with the 0.1 snap (`:50444-50464`).
    #[test]
    fn s1_d_4b_12_the_boulder_pet_is_rng_free_and_settles_near_home() {
        let w = world();
        let mut p = player(&w);
        let (cx, cy) = p.center();
        // The home spot is cx − 25; standing on it keeps |num51| < 50.
        let mut proj = pet(BOULDER_PET, 32.0, 32.0, Vec2::new(cx - 41.0, cy - 16.0));
        let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
        proj.vel = Vec2::new(0.5, 0.0);
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        assert_eq!(census.floats, 0, "the boulder draws nothing");
        assert!(census.ranges.is_empty());
        // 0.5 * 0.9 = 0.45 — above the 0.1 snap, below the 0.3 spin gate.
        assert!((proj.vel.x - 0.45).abs() < 1e-6, "the 0.9 bleed");
        // The per-tick spin (`:49492`) ran before the settle: 0.5/16.
        assert!((proj.rot - 0.5 / 16.0).abs() < 1e-6, "`rotation += velocity.X / 16`");
        // A second tick: 0.45 * 0.9 = 0.405, still above the snap.
        proj.vel.y = 0.0;
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        assert!((proj.vel.x - 0.405).abs() < 1e-6);
        assert_eq!(census.floats, 0);
    }

    /// `:50271-50300` with the pets' `num3 = 200` (`:49330`/`:49354`/
    /// `:49366`): 201 px away flips to state 1, 199 px stays walking.
    #[test]
    fn s1_d_4b_12_the_pets_fly_home_past_200_px() {
        let w = world();
        for kind in [CRIMSON_HEART, DIRTIEST_BLOCK, BOULDER_PET] {
            let mut p = player(&w);
            let (cx, cy) = p.center();
            let mut proj = pet(kind, 16.0, 16.0, Vec2::new(cx + 201.0 - 8.0, cy - 8.0));
            let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
            proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
            assert_eq!(proj.ai[0], 1.0, "kind {kind}: 201 px flies home");
            let mut proj = pet(kind, 16.0, 16.0, Vec2::new(cx + 199.0 - 8.0, cy - 8.0));
            proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
            assert_eq!(proj.ai[0], 0.0, "kind {kind}: 199 px keeps walking");
        }
    }

    /// `:49771-49779` — the state-1 landing: close, the owner grounded, the
    /// pet's bottom at or above the owner's and no solid box → state 0 with
    /// the −6 velocity clamp.
    #[test]
    fn s1_d_4b_12_the_flight_lands_below_the_owners_feet() {
        let w = world();
        let mut p = player(&w);
        let (cx, cy) = p.center();
        let mut proj = pet(BOULDER_PET, 32.0, 32.0, Vec2::new(cx - 16.0, cy - 16.0));
        proj.ai[0] = 1.0;
        proj.vel = Vec2::new(0.0, -9.0);
        let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        assert_eq!(proj.ai[0], 0.0, "the landing");
        // The clamp (`:49775-49778`) then the SAME tick's walk gravity
        // (`:51144`, the landing flips `ai[0]` to 0 before the walk block):
        // the ground-dropped aim sits >100 px below and vel.y = -6, so both
        // `GetLerpValue` factors clamp to 1 (`:50607`) — `num56 = 1`.
        assert!(
            (proj.vel.y - (-6.0 + 0.4 + 1.0)).abs() < 1e-6,
            "the -6 clamp + the walk gravity with the fall-through lift"
        );
        assert!(proj.tile_collide, "the same-tick walk re-enables collide (`:50365`)");
    }

    /// W4-d7 F3 — the Companion Cube's lava scream (`:49506-49524`): the
    /// localAI[0] ramps in lava, kills at 20, the scream cue on the cooldown
    /// reset (the Next(10) pick). The centre tile's lava drives the ramp.
    #[test]
    fn w4_d7_f3_the_companion_cube_lava_scream() {
        let mut w = world();
        // A lava tile at the cube's centre (tile 20, 20 → px 320, 320).
        let i = w.index(20, 20);
        w.tiles[i].liquid = 255;
        w.tiles[i].set_liquid_kind(tr_core::tile::LiquidKind::Lava);
        let mut proj = pet(COMPANION_CUBE, 30.0, 30.0, Vec2::new(305.0, 305.0));
        proj.local_ai[0] = 19.0; // one tick from the scream
        let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
        proj.companion_cube_tick(&w, None, &mut census, None);
        assert!(!proj.active, "the 20th lava tick kills");
        assert_eq!(proj.local_ai[2], 3600.0, "the scream cooldown reset");
        assert_eq!(census.ranges, vec![(0, 10)], "the Next(10) scream pick");
        assert!(proj.wants_sound.is_some(), "the scream cue");
    }

    /// W4-d7 F3 — the Companion Cube's darkness bite (`:49525-49554`): the
    /// localAI[1] ramps in the dark, the Next(30,120) threshold draw runs
    /// FIRST, the 1/5 fake-scream vs the bite. A lit snapshot folds it inert.
    #[test]
    fn w4_d7_f3_the_companion_cube_darkness_bite() {
        let w = world();
        let mut p = player(&w);
        p.vx = 0.0;
        p.vy = 0.0;
        // immune=true blocks the bite AFTER the threshold draw (the C# &&
        // short-circuit: the Next(30,120) draws first, then !player.immune).
        p.immune = true;
        let mut proj = pet(COMPANION_CUBE, 30.0, 30.0, Vec2::new(305.0, 305.0));
        proj.local_ai[1] = 119.0; // one tick from the 120 cap
        let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
        // Dark at both samples: the ramp climbs, the threshold draw runs.
        proj.companion_cube_tick(&w, Some(&mut p), &mut census, Some((0.0, 0.0)));
        assert_eq!(proj.local_ai[1], 120.0, "the dark ramp clamps at 120");
        assert_eq!(census.ranges, vec![(30, 120)], "the threshold draw runs first, the immune gate blocks the bite");
        // A lit snapshot: no ramp, no threshold draw.
        let mut proj2 = pet(COMPANION_CUBE, 30.0, 30.0, Vec2::new(305.0, 305.0));
        proj2.local_ai[1] = 50.0;
        let mut census2 = CensusRng { ranges: Vec::new(), floats: 0 };
        proj2.companion_cube_tick(&w, Some(&mut p), &mut census2, Some((1.0, 1.0)));
        assert_eq!(proj2.local_ai[1], 49.0, "the lit decay");
        assert!(census2.ranges.is_empty(), "no threshold draw when lit");
    }

    /// W4-d7 F3 — the Rainbow Boulder Pet's flight steering (`:49807-49819`):
    /// the far-arm RotatedByRandom(0.5236) draws TWO NextDouble (the f64 seam),
    /// the close-arm renormalizes to num18=18. The shimmer particle's
    /// miscCounter%2 gate draws ONE NextFloat.
    #[test]
    fn w4_d7_f3_the_rainbow_boulder_flight_steering() {
        let w = world();
        let mut p = player(&w);
        let (cx, cy) = p.center();
        // Far from the owner (>350): the RotatedByRandom arm.
        let mut proj = pet(RAINBOW_BOULDER_PET, 32.0, 32.0, Vec2::new(cx + 500.0, cy));
        proj.ai[0] = 1.0;
        p.misc_counter = 1; // odd → no shimmer draw
        let mut rng = tr_core::UnifiedRandom::new(1090);
        let mut oracle = tr_core::UnifiedRandom::new(1090);
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut rng, None);
        // The two RotatedByRandom draws (the f64 seam).
        let _ = oracle.next_double();
        let _ = oracle.next_double();
        assert_eq!(rng.next_range(0, 1), oracle.next_range(0, 1), "exactly two draws");
        assert_eq!(proj.local_ai[0], 1.0, "the bounce-particle latch");
    }

    /// W4-d7 F3 — the Companion Cube's bite (`:49547-49551`): Hurt(3) with
    /// hitDirection 0 — NO knockback pop (`Player.cs:38519`'s
    /// `hitDirection != 0` gate), the i-frame clear INCLUDING the port's
    /// collapsed `hurt_cd`, localAI[1] = -300 + Next(30)*-10.
    #[test]
    fn w4_d7_f3_the_companion_cube_bite_takes_no_pop() {
        // A scripted stream: the threshold 30, the Next(5) bite pick 1
        // (non-zero → the bite, not the fake scream), the Next(30) cooldown 0.
        struct ScriptRng {
            rolls: std::collections::VecDeque<i32>,
        }
        impl DropRng for ScriptRng {
            fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
                self.rolls.pop_front().unwrap()
            }
            fn next_float(&mut self) -> f32 {
                0.0
            }
        }
        let w = world();
        let mut p = player(&w);
        p.vx = 0.0;
        p.vy = 0.0;
        p.immune = false;
        p.hurt_cd = 0.0;
        let hp = p.hp;
        let mut proj = pet(COMPANION_CUBE, 30.0, 30.0, Vec2::new(305.0, 305.0));
        proj.local_ai[1] = 119.0;
        let mut rng = ScriptRng { rolls: [30, 1, 0].into_iter().collect() };
        proj.companion_cube_tick(&w, Some(&mut p), &mut rng, Some((0.0, 0.0)));
        assert_eq!(p.hp, hp - 3, "the 3-damage bite (no defense)");
        assert_eq!((p.vx, p.vy), (0.0, 0.0), "hitDirection 0 — no pop");
        assert!(!p.immune, "the i-frame clear");
        assert_eq!(p.immune_time, 0);
        assert_eq!(p.hurt_cd, 0.0, "the collapsed cooldown clears too");
        assert_eq!(proj.local_ai[1], -300.0, "-300 + Next(30)*-10 with the 0 roll");
        assert_eq!(proj.wants_sound, Some((2, 1, 320.0, 320.0, 0.0, 1.0)), "Item1 at the centre");
    }

    /// W4-d7 F3 — the 1090's frame cycle + localAI[2] fade (`:51172-51200`):
    /// the 2-tick frame counter (the 100-wrap), the ±0.025 ramp keyed on
    /// ai[0] == 1. Below the owner's feet the landing never fires, so the
    /// flight state holds; the close arm draws nothing.
    #[test]
    fn w4_d7_f3_the_rainbow_boulder_frame_cycle_and_fade() {
        let w = world();
        let mut p = player(&w);
        let (cx, cy) = p.center();
        // Below the owner's feet (no landing), inside the 350 close radius.
        let mut proj = pet(RAINBOW_BOULDER_PET, 32.0, 32.0, Vec2::new(cx - 16.0, cy + 40.0));
        proj.ai[0] = 1.0;
        proj.local_ai[2] = 0.5;
        p.misc_counter = 1; // odd → no shimmer draw
        let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        assert_eq!(proj.frame_counter, 1, "tick 1: the counter climbs");
        assert_eq!(proj.frame, 0);
        proj.ai_freaking_pirates_pets(&w, Some(&mut p), &mut census, None);
        assert_eq!(proj.frame_counter, 0, "tick 2: the 2-tick wrap");
        assert_eq!(proj.frame, 1);
        assert!((proj.local_ai[2] - 0.55).abs() < 1e-6, "the +0.025 flight ramp");
        assert_eq!(census.floats, 0, "the close arm + odd miscCounter draw nothing");
        assert!(census.ranges.is_empty());
    }

    /// W4-d7 F3 — the 1036's `Kill()` dust census (`:71068-71076`): 3× dust 5,
    /// each ONE `NextVector2Circular(4,4)` = TWO NextFloat draws — six floats,
    /// no integer ranges.
    #[test]
    fn w4_d7_f3_the_cob_whip_spider_kill_census() {
        let mut census = CensusRng { ranges: Vec::new(), floats: 0 };
        cob_whip_spider_kill_draws(&mut census);
        assert_eq!(census.floats, 6, "two draws per dust, three dusts");
        assert!(census.ranges.is_empty());
    }
}
