//! The aiStyle-101 family — the Ancient Doom (`NPC.AI()`'s
//! `else if (this.aiStyle == 101)` arm, `NPC.cs:40067-40146`). W4-d33,
//! research/25 row 101, CODE-ONLY MODE (no compiler invoked; the
//! witness tests below are written, not run).
//!
//! ## Census
//!
//! The family has exactly ONE vanilla member: the AncientDoom 523
//! (`NPCID.AncientDoom`, `SetDefaults` row `NPC.cs:15795-15812`). The
//! census of `this.aiStyle = 101;` across the decompiled tree returns
//! three hits: `NPC.cs:15799` (the `type == 523` branch) and
//! `Projectile.cs:6111`/`:6401` (projectile space, out of NPC scope).
//! The port's roster has no `NpcKind::AncientDoom` — the module lands
//! UNWIRED (the `npc_ai_style73.rs` precedent), keyed on the vanilla
//! `npc_type` until the roster wave.
//!
//! ## The machine
//!
//! The cultist's summoned doom orb: a 420-tick fuse growing from
//! invisible to full size, anchored to the Lunatic Cultist (439) in
//! the NPC slot `ai[0]`. The anchor gate (`:40075-40090`):
//! `flag109 = !(ai[1] >= 0) || !anchor.active`; a NON-439 anchor also
//! sets it (the orb dies when its cultist is replaced); a 439 anchor
//! below half life doubles the growth step (`num1374 = 2`), below a
//! quarter triples it (`= 3`) — both INTEGER `lifeMax / 2` divisions.
//!
//! The growth (`:40091-40096`): `ai[1] += step`;
//! `t = Clamp(ai[1] / 120, 0, 1)`; the `position = Center; scale =
//! Lerp(0, 1, t); Center = position` round trip (an identity for the
//! box — scale does not resize it — ported verbatim anyway);
//! `alpha = (int)(255 - t*255)`.
//!
//! The three dust gates (`:40097-40126`), each `Next(6) == 0`:
//! 1. type 27 at `Center - v*20` where `v = UnitY.RotatedByRandom(2π)`
//!    (TWO `NextDouble`); `noGravity`; position re-seeded to
//!    `Center - v*Next(10,21)*scale`; velocity `v.RotatedBy(π/2)*4`;
//!    `scale = 0.5 + NextFloat()`; `fadeIn = 0.5`.
//! 2. type 240 at `Center - v*30`; `noGravity`; position
//!    `Center - v*20*scale`; velocity `v.RotatedBy(-π/2)*2`;
//!    `scale = 0.5 + NextFloat()`; `fadeIn = 0.5`.
//! 3. type 240 at `Center - v*30`; position `Center - v*20*scale`;
//!    velocity ZERO; `scale = 0.5 + NextFloat()`; `fadeIn = 0.5`;
//!    `noLight`.
//!
//! The shimmer pair (`:40127-40128`): `localAI[0] += π/60`;
//! `localAI[1] = 0.25 + UnitY.RotatedBy(ai[1]*2π/60).Y * 0.25` (the
//! draw-layer's pulse inputs — no draws here).
//!
//! The fuse (`:40129-40140`): at `ai[1] >= 420` the death flag sets
//! and, on a non-client (`netMode != 1`), FOUR type-593
//! (`ProjectileID.AncientDoomProjectile`) bolts fire from the centre
//! at `(0, -4).RotatedBy(π/2 * i)` for i in 0..4 — the cardinal cross,
//! `this.damage`, knockback 0, owner `Main.myPlayer`. The death
//! (`:40141-40145`): `HitEffect(0, 9999.0)` + `active = false`.
//!
//! ## RNG order (by construction)
//!
//! Per tick: three `Next(6)` gates in order; each passing gate draws
//! the `RotatedByRandom` pair (2 `NextDouble`), the `NewDust`
//! internals, then (gate 1 only) `Next(10, 21)`, then one
//! `NextFloat`. The anchor reads, the growth maths, the shimmer pair
//! and the four bolts draw NOTHING.
//!
//! `HitEffect(0, 9999.0)` for 523 is a NO-OP: no type arm in the whole
//! `NPC.cs:81654-89834` chain matches 523 (the standalone ifs, the
//! 552-family chain, the 516/519/522 chain, the gold-critter gate and
//! the generic else-chain all miss; the final else returns at
//! `:89813-89815`), so it spawns nothing and draws nothing. The row's
//! `DeathSound = NPCDeath6` is NOT played on this path — the arm sets
//! `active = false` directly, never `checkDead`.
//!
//! ## FindFrame case 523
//!
//! `NPC.cs:72127-72136`: `++frameCounter >= 6` advances the row and
//! zeroes the counter, wrapping at `Main.npcFrameCount[523] = 5`.
//! [`doom_find_frame`] ports it as the pure pair function (the
//! style73 `turret_find_frame` convention).
#![allow(dead_code)] // wiring deferred — the kind + dispatch arm land with the roster wave

use crate::npc::Npc;
use crate::npc_ai_style37::rotated_by;
use glam::Vec2;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The type-523 `SetDefaults` row (`NPC.cs:15795-15812`), verbatim.
/// The row writes `alpha = 0` then `alpha = 255` (`:15807`/`:15810`)
/// — the final 255 wins.
pub struct AncientDoomDefaults {
    pub width: i32,
    pub height: i32,
    pub ai_style: i32,
    pub damage: i32,
    pub defense: i32,
    pub life_max: i32,
    /// `(SoundType, Style)` — `SoundID.NPCHit1`.
    pub hit_sound: (i32, i32),
    /// `(SoundType, Style)` — `SoundID.NPCDeath6`.
    pub death_sound: (i32, i32),
    pub knock_back_resist: f32,
    pub no_gravity: bool,
    pub no_tile_collide: bool,
    pub alpha: u8,
    pub chaseable: bool,
    pub can_ghost_heal: bool,
}

/// `NPC.cs:15795-15812` — `else if (this.type == 523)`.
pub const ANCIENT_DOOM: AncientDoomDefaults = AncientDoomDefaults {
    width: 42,
    height: 42,
    ai_style: 101,
    damage: 30,
    defense: 0,
    life_max: 500,
    hit_sound: (3, 1),
    death_sound: (4, 6),
    knock_back_resist: 0.0,
    no_gravity: true,
    no_tile_collide: true,
    alpha: 255,
    chaseable: false,
    can_ghost_heal: false,
};

/// `Main.npcFrameCount[523]` = 5 (the 520-529 row).
pub const ANCIENT_DOOM_FRAME_COUNT: u32 = 5;

/// The anchor slot view — `Main.npc[(int)this.ai[0]]` as the arm
/// reads it (`:40075-40086`). The wiring caller fills it from the
/// pool slot; the pure module never indexes the pool itself.
#[derive(Clone, Copy, Debug)]
pub struct DoomAnchor {
    /// `Main.npc[slot].active`.
    pub active: bool,
    /// `Main.npc[slot].type` — only 439 (the Lunatic Cultist) keeps
    /// the orb alive.
    pub npc_type: i32,
    /// `Main.npc[slot].life`.
    pub life: i32,
    /// `Main.npc[slot].lifeMax`.
    pub life_max: i32,
}

/// The per-tick environment (the `Style86Env` shape).
pub struct Style101Env<'a> {
    pub world: &'a World,
    pub dust: &'a mut DustPool,
    pub dust_ctx: &'a SpawnCtx,
    pub rng: &'a mut UnifiedRandom,
    /// The `ai[0]` slot's view (the wiring seam — module docs).
    pub anchor: DoomAnchor,
    /// `Main.netMode` (0 single-player, 1 client, 2 server) — the
    /// fuse bolts fire on `!= 1` (`:40132`).
    pub net_mode: i32,
    /// `this.damage` at the call — the SetDefaults 30 unless the
    /// spawn scaled it (the wiring caller's value).
    pub damage: i32,
}

/// The `Projectile.NewProjectile` request of the fuse (`:40137`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DoomBolt {
    /// `ProjectileID.AncientDoomProjectile` = 593.
    pub projectile_type: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
}

/// The arm's per-tick side effects beyond the NPC state.
#[derive(Default, Debug, PartialEq)]
pub struct Style101Outcome {
    /// The death flag (`:40141-40145`): `HitEffect(0, 9999.0)` is the
    /// documented no-op (module docs); the caller owes only the
    /// `active = false` deactivation.
    pub died: bool,
    /// The fuse's four cardinal bolts (`:40134-40138`), in i order.
    /// Empty on every other tick.
    pub shots: Vec<DoomBolt>,
}

/// `Utils.RotatedByRandom(maxRadians)` (`Utils.cs:1697-1700`): TWO
/// `NextDouble` draws, `+first×max − second×max`.
fn rotated_by_random(v: Vec2, max_radians: f64, rng: &mut UnifiedRandom) -> Vec2 {
    let a = rng.next_double() * max_radians;
    let b = rng.next_double() * max_radians;
    let (x, y) = rotated_by(v.x, v.y, a - b, 0.0, 0.0);
    Vec2::new(x, y)
}

/// One fixed step of the aiStyle-101 arm (`NPC.cs:40067-40146`), in
/// C# order. The caller owns the raw movement integration after
/// `AI()` (the row's `noGravity`/`noTileCollide` are both true) and
/// drains [`Style101Outcome`].
pub fn style101_step(npc: &mut Npc, env: &mut Style101Env<'_>) -> Style101Outcome {
    let mut out = Style101Outcome::default();
    // `:40069-40074` — the C# locals, verbatim.
    let fuse = 420.0f32; // num1372
    let fade = 120.0f32; // num1373
    let mut step = 1.0f32; // num1374
    let shot_speed = 4.0f32; // num1375
    // `:40075-40090` — the anchor gate.
    let mut dies = !(npc.ai[1] >= 0.0) || !env.anchor.active;
    if env.anchor.npc_type == 439 {
        if env.anchor.life < env.anchor.life_max / 2 {
            step = 2.0;
        }
        if env.anchor.life < env.anchor.life_max / 4 {
            step = 3.0;
        }
    } else {
        dies = true;
    }
    // `:40091-40096` — the growth + the Centre round trip.
    npc.ai[1] += step;
    let t = (npc.ai[1] / fade).clamp(0.0, 1.0);
    let (cx, cy) = npc.center();
    npc.x = cx;
    npc.y = cy;
    npc.scale = t; // Lerp(0, 1, t)
    let (w, h) = npc.size();
    npc.x = cx - w / 2.0; // the Center setter
    npc.y = cy - h / 2.0;
    npc.alpha = (255.0 - t * 255.0) as u8;
    let (cx, cy) = npc.center();
    // `:40097-40106` — dust gate 1 (type 27).
    if env.rng.next_max(6) == 0 {
        let v = rotated_by_random(Vec2::new(0.0, 1.0), 2.0 * std::f64::consts::PI, env.rng);
        let idx = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            Vec2::new(cx - v.x * 20.0, cy - v.y * 20.0),
            0,
            0,
            27,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        {
            let d = env.dust.slot_mut(idx);
            d.no_gravity = true;
            let r = env.rng.next_range(10, 21) as f32;
            d.position = Vec2::new(cx - v.x * r * npc.scale, cy - v.y * r * npc.scale);
            let (rx, ry) = rotated_by(v.x, v.y, std::f64::consts::FRAC_PI_2, 0.0, 0.0);
            d.velocity = Vec2::new(rx * 4.0, ry * 4.0);
            d.scale = 0.5 + env.rng.next_float();
            d.fade_in = 0.5;
        }
    }
    // `:40107-40116` — dust gate 2 (type 240, the -π/2 spin).
    if env.rng.next_max(6) == 0 {
        let v = rotated_by_random(Vec2::new(0.0, 1.0), 2.0 * std::f64::consts::PI, env.rng);
        let idx = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            Vec2::new(cx - v.x * 30.0, cy - v.y * 30.0),
            0,
            0,
            240,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        {
            let d = env.dust.slot_mut(idx);
            d.no_gravity = true;
            d.position = Vec2::new(cx - v.x * 20.0 * npc.scale, cy - v.y * 20.0 * npc.scale);
            let (rx, ry) = rotated_by(v.x, v.y, -std::f64::consts::FRAC_PI_2, 0.0, 0.0);
            d.velocity = Vec2::new(rx * 2.0, ry * 2.0);
            d.scale = 0.5 + env.rng.next_float();
            d.fade_in = 0.5;
        }
    }
    // `:40117-40126` — dust gate 3 (type 240, still, noLight).
    if env.rng.next_max(6) == 0 {
        let v = rotated_by_random(Vec2::new(0.0, 1.0), 2.0 * std::f64::consts::PI, env.rng);
        let idx = env.dust.new_dust(
            env.rng,
            env.dust_ctx,
            env.world,
            Vec2::new(cx - v.x * 30.0, cy - v.y * 30.0),
            0,
            0,
            240,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        {
            let d = env.dust.slot_mut(idx);
            d.position = Vec2::new(cx - v.x * 20.0 * npc.scale, cy - v.y * 20.0 * npc.scale);
            d.velocity = Vec2::ZERO;
            d.scale = 0.5 + env.rng.next_float();
            d.fade_in = 0.5;
            d.no_light = true;
        }
    }
    // `:40127-40128` — the shimmer pair (the draw layer's inputs).
    npc.local_ai[0] += std::f32::consts::PI / 60.0;
    let angle = (npc.ai[1] * (std::f32::consts::PI * 2.0) / 60.0) as f64;
    let (_, uy) = rotated_by(0.0, 1.0, angle, 0.0, 0.0);
    npc.local_ai[1] = 0.25 + uy * 0.25;
    // `:40129-40140` — the fuse + the cardinal cross.
    if npc.ai[1] >= fuse {
        dies = true;
        if env.net_mode != 1 {
            for i in 0..4 {
                let a = (std::f32::consts::FRAC_PI_2 * i as f32) as f64;
                let (vx, vy) = rotated_by(0.0, -shot_speed, a, 0.0, 0.0);
                out.shots.push(DoomBolt {
                    projectile_type: 593,
                    x: cx,
                    y: cy,
                    vx,
                    vy,
                    damage: env.damage,
                });
            }
        }
    }
    // `:40141-40145` — HitEffect(0, 9999.0) is the 523 no-op (module
    // docs); the deactivation is the caller's seam.
    if dies {
        out.died = true;
    }
    out
}

/// `NPC.FindFrame` case 523 (`NPC.cs:72127-72136`): the 6-tick row
/// advance over the 5-row sheet (the style73 `turret_find_frame`
/// convention — `frame_row` is `frame.Y / num`).
pub fn doom_find_frame(frame_row: u32, frame_counter: f32) -> (u32, f32) {
    let mut counter = frame_counter + 1.0; // `++this.frameCounter`
    let mut row = frame_row;
    if counter >= 6.0 {
        row += 1;
        counter = 0.0;
        if row >= ANCIENT_DOOM_FRAME_COUNT {
            row = 0;
        }
    }
    (row, counter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    /// A 523-shaped NPC (the style73 placeholder convention).
    fn doom(x: f32, y: f32) -> Npc {
        let d = &ANCIENT_DOOM;
        let mut n = Npc::new(NpcKind::Zombie, x, y, 1, 0.0);
        n.size_override = Some((d.width as f32, d.height as f32));
        n.hp = d.life_max;
        n.max_hp = d.life_max;
        n.alpha = d.alpha;
        n
    }

    fn cultist(life: i32) -> DoomAnchor {
        DoomAnchor { active: true, npc_type: 439, life, life_max: 32000 }
    }

    fn air() -> World {
        World::new("t".into(), 400, 300)
    }

    fn spawn_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    fn menu_ctx() -> SpawnCtx {
        SpawnCtx { game_menu: true, ..spawn_ctx() }
    }

    /// `NPC.cs:15795-15812` — the SetDefaults row, field by field.
    #[test]
    fn w4_d33_s101_the_defaults_row() {
        let d = &ANCIENT_DOOM;
        assert_eq!((d.width, d.height), (42, 42));
        assert_eq!(d.ai_style, 101);
        assert_eq!((d.damage, d.defense, d.life_max), (30, 0, 500));
        assert_eq!(d.hit_sound, (3, 1));
        assert_eq!(d.death_sound, (4, 6));
        assert_eq!(d.alpha, 255, "the second alpha write wins");
        assert!(!d.chaseable);
        assert!(!d.can_ghost_heal);
        assert_eq!(ANCIENT_DOOM_FRAME_COUNT, 5);
    }

    /// `:40075-40091` — the anchor step ladder: full life steps 1,
    /// below half steps 2, below quarter steps 3 (the INTEGER
    /// lifeMax divisions).
    #[test]
    fn w4_d33_s101_the_anchor_life_scales_the_growth_step() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        for (life, want) in [(32000, 1.0f32), (15999, 2.0), (7999, 3.0)] {
            let mut rng = UnifiedRandom::new(101);
            let mut n = doom(100.0, 100.0);
            let mut env = Style101Env {
                world: &w,
                dust: &mut pool,
                dust_ctx: &c,
                rng: &mut rng,
                anchor: cultist(life),
                net_mode: 0,
                damage: 30,
            };
            let out = style101_step(&mut n, &mut env);
            assert_eq!(n.ai[1], want, "life {life}");
            assert!(!out.died);
        }
    }

    /// `:40075-40090` + `:40141-40145` — a dead anchor, a non-439
    /// anchor and a negative counter each kill the orb (the growth
    /// still runs first).
    #[test]
    fn w4_d33_s101_the_death_gates() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(101);
        let mut n = doom(100.0, 100.0);
        let mut env = Style101Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            anchor: DoomAnchor { active: false, npc_type: 439, life: 32000, life_max: 32000 },
            net_mode: 0,
            damage: 30,
        };
        assert!(style101_step(&mut n, &mut env).died, "the inactive anchor");
        let mut n = doom(100.0, 100.0);
        env.anchor = DoomAnchor { active: true, npc_type: 440, life: 32000, life_max: 32000 };
        assert!(style101_step(&mut n, &mut env).died, "the non-439 anchor");
        let mut n = doom(100.0, 100.0);
        n.ai[1] = -5.0;
        env.anchor = cultist(32000);
        assert!(style101_step(&mut n, &mut env).died, "the negative counter");
    }

    /// `:40091-40096` — the fade ramp: at ai[1] = 60 the scale is 0.5
    /// and alpha 127; the Centre round trip leaves the box in place.
    #[test]
    fn w4_d33_s101_the_fade_ramp_and_the_centre_round_trip() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(101);
        let mut n = doom(100.0, 100.0);
        n.ai[1] = 59.0;
        let mut env = Style101Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            anchor: cultist(32000),
            net_mode: 0,
            damage: 30,
        };
        let _ = style101_step(&mut n, &mut env);
        assert_eq!(n.ai[1], 60.0);
        assert_eq!(n.scale, 0.5);
        assert_eq!(n.alpha, 127, "(int)(255 - 0.5*255)");
        assert_eq!((n.x, n.y), (100.0, 100.0), "the identity round trip");
    }

    /// `:40097-40126` — the three-gate draw census under the menu
    /// ctx: each passing gate draws 2 NextDouble + (gate 1 only)
    /// Next(10,21) + NextFloat. The mirrored oracle replays it.
    #[test]
    fn w4_d33_s101_the_three_gates_draw_in_order() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(101);
        let mut oracle = UnifiedRandom::new(101);
        let mut n = doom(100.0, 100.0);
        let mut env = Style101Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            anchor: cultist(32000),
            net_mode: 0,
            damage: 30,
        };
        let _ = style101_step(&mut n, &mut env);
        for gate in 0..3 {
            if oracle.next_max(6) == 0 {
                oracle.next_double();
                oracle.next_double();
                if gate == 0 {
                    oracle.next_range(10, 21);
                }
                oracle.next_float();
            }
        }
        assert_eq!(rng.peek(), oracle.peek(), "the gate draw order");
    }

    /// `:40129-40140` — the fuse fires the four cardinal bolts (up,
    /// right, down, left — the (0,-4) spin) and sets the death flag;
    /// a client (netMode 1) dies WITHOUT bolts.
    #[test]
    fn w4_d33_s101_the_fuse_fires_the_cardinal_cross() {
        let w = air();
        let mut pool = DustPool::new();
        let c = menu_ctx();
        let mut rng = UnifiedRandom::new(101);
        let mut n = doom(100.0, 100.0);
        n.ai[1] = 419.0;
        let mut env = Style101Env {
            world: &w,
            dust: &mut pool,
            dust_ctx: &c,
            rng: &mut rng,
            anchor: cultist(32000),
            net_mode: 0,
            damage: 30,
        };
        let out = style101_step(&mut n, &mut env);
        assert!(out.died);
        assert_eq!(out.shots.len(), 4);
        assert!(out.shots.iter().all(|s| s.projectile_type == 593 && s.damage == 30));
        // i=0: (0,-4); i=1: (4,0); i=2: (0,4); i=3: (-4,0) (+-eps).
        let want = [(0.0, -4.0), (4.0, 0.0), (0.0, 4.0), (-4.0, 0.0)];
        for (s, (wx, wy)) in out.shots.iter().zip(want) {
            assert!((s.vx - wx).abs() < 1e-4 && (s.vy - wy).abs() < 1e-4);
        }
        let mut n = doom(100.0, 100.0);
        n.ai[1] = 419.0;
        env.net_mode = 1;
        let out = style101_step(&mut n, &mut env);
        assert!(out.died);
        assert!(out.shots.is_empty(), "the client never fires");
    }

    /// `NPC.cs:72127-72136` — the 6-tick row advance wraps at 5.
    #[test]
    fn w4_d33_s101_the_find_frame_wraps_at_five() {
        let (row, counter) = doom_find_frame(0, 4.0);
        assert_eq!((row, counter), (0, 5.0));
        let (row, counter) = doom_find_frame(0, 5.0);
        assert_eq!((row, counter), (1, 0.0));
        let (row, counter) = doom_find_frame(4, 5.0);
        assert_eq!((row, counter), (0, 0.0), "the wrap");
    }

    /// The full-pool stream parity: 30 ticks on the REAL dust ctx
    /// must match a cloned twin step bit-for-bit (the style86
    /// twin-pool precedent).
    #[test]
    fn w4_d33_s101_the_full_pool_run_is_twin_stable() {
        let w = air();
        let c = spawn_ctx();
        let mut a = doom(600.0, 600.0);
        let mut b = a.clone();
        let mut pool_a = DustPool::new();
        let mut pool_b = DustPool::new();
        let mut rng_a = UnifiedRandom::new(523);
        let mut rng_b = UnifiedRandom::new(523);
        for tick in 0..30 {
            let mut env_a = Style101Env {
                world: &w, dust: &mut pool_a, dust_ctx: &c, rng: &mut rng_a,
                anchor: cultist(32000), net_mode: 0, damage: 30,
            };
            let _ = style101_step(&mut a, &mut env_a);
            let mut env_b = Style101Env {
                world: &w, dust: &mut pool_b, dust_ctx: &c, rng: &mut rng_b,
                anchor: cultist(32000), net_mode: 0, damage: 30,
            };
            let _ = style101_step(&mut b, &mut env_b);
            assert_eq!(rng_a.peek(), rng_b.peek(), "stream @{tick}");
            assert_eq!(a.scale.to_bits(), b.scale.to_bits(), "scale @{tick}");
            assert_eq!(a.local_ai[1].to_bits(), b.local_ai[1].to_bits(), "pulse @{tick}");
        }
    }
}
