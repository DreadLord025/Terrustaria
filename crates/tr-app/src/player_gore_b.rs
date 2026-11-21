//! Player-side `Gore.NewGore*` sites, wave B: the dash launch + the
//! sandstorm hold visuals + the team-hair-dye sparkle
//! (`Player.cs:21034-24294`). W4-g17 wiring wave: the dash==1 tail is
//! LIVE — the `player.rs` dash start queues `pending_dash_gores` and
//! `world_sim::player`'s drain runs [`dash_launch_gores`] on the
//! `UpdatePlayers` stream. The sandstorm hold ([`sandstorm_jump_gore`])
//! and the hair-dye arms stay unwired: the port has no
//! `DoubleJumpVisuals` dispatch yet and no team-dye state
//! (`Player.hairDye` is unported — the dye cache's recorded deviation),
//! so their claims wait for those seams (the `npc_hit_effect_a.rs`
//! precedent).
//!
//! ## Scope — this commit's sites (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1-2 | `:21427`/`:21431` | `DashMovement` dash==1 — 2× Next(61,64) |
//! | 3 | `:21922` | `DoubleJumpVisuals` sandstorm — Next(220,223) |
//! | 4 | `:24285` | `UpdateHairDyeDust` — Next(276,283) |
//!
//! ## RNG contract (the player tick's `Main.rand` stream)
//!
//! * Dash: 20 dusts first — per dust the `NewDust` internals (type 31
//!   hits no tweak: 6 draws) then the position jitter `Next(-5,6)` ×2
//!   and the scale roll `Next(20)` (`:21422-21425`). Then per gore:
//!   `Next(61,64)` type (argument), the kicks (2, pool-side), the
//!   `Next(-50,51)` ×2 velocity OVERWRITE (the kicks are discarded),
//!   then the ×0.4 scale. The velocity writes are `X = roll*0.01`,
//!   `Y = roll*0.01` — NOT additive.
//! * Sandstorm: 3 type-124 dusts first (6 draws each; the `num4`
//!   scale is data), then the `miscCounter % 3 == 0` gate (no draw)
//!   and ONE gore: `Next(220,223)` type, kicks (2), then the velocity
//!   OVERWRITE `playerVel * 0.3 * num4` + alpha 100.
//! * Hair dye: the `Next(45)` dust gate ALWAYS draws; the `Next(225)`
//!   gore gate ALWAYS draws (the C# runs both ifs unconditionally).
//!   When the gore gate hits: `Next(276,283)` type, the position
//!   `Next(width)` + `Next(8)` (2), kicks (2), then the six post
//!   rolls: X/Y `Next(-50,51)` multipliers, scale `Next(-20,21)`,
//!   X/Y `Next(-50,51)` adds, the Y decrement and the `+vel*0.2`
//!   (the last two are arithmetic).
//!
//! The post-spawn writes land on the returned slot, the sentinel
//! included (the `npc_hit_effect_a.rs` convention).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * Dash: the `dash == 1` gate, `DoCommonDashHandle`, the 16.9
//!   velocity write, the solid-tile halving probe and `dashDelay = -1`
//!   are the movement lane's — this module runs the visual tail of a
//!   COMMITTED dash. The dust loop's `shader` write is a dropped
//!   field (the pool's convention).
//! * Sandstorm: the `isPerformingJump_Sandstorm && hasJumpOption &&
//!   !canJumpAgain && ascending` gate (`:21905`) is the caller's; the
//!   `sandStorm = true` flag write is RETURNED-implicit (the caller
//!   owns the flag). `jump` rides the state (the `num4` scale reads
//!   it). The reversed-gravity height pick is -6 (`:21909-21910`).
//! * Hair dye: the `netMode != 2 && !gamePaused && !dead && !ghost &&
//!   !stoned && !frozen && hairDye == TeamDyeShaderIndex` gate
//!   (`:24270`) is the caller's — the team-dye shader id resolves
//!   from item 1984 (`ID/ContentSamples.cs:19`). The dust arm's draws
//!   are consumed here ([`hair_dye_dust_arm`]) so the wiring can run
//!   both arms on one stream in C# order.
//!
//! See docs/ANALYSIS.md "The player gore sites (W4-g17, wave B)".
#![allow(dead_code)] // the sandstorm/hair-dye arms wait for their dispatch seams; the dash tail is live

use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The player snapshot the dash visuals read — the POST-launch state
/// (the 16.9 velocity write already ran, `:21411`).
#[derive(Clone, Copy, Debug)]
pub struct DashGoreState {
    pub x: f32,
    pub y: f32,
    /// C# int hitbox (`Entity.cs:22-26`).
    pub width: i32,
    pub height: i32,
}

/// `Player.DashMovement`'s dash==1 visual tail (`Player.cs:21419-21434`):
/// the 20 type-31 jitter dusts then the 2 gores (sites 1-2) at
/// centre-(24,34) / centre-(24,14), types `Next(61,64)`, the velocity
/// OVERWRITE `Next(-50,51)*0.01` per axis then ×0.4.
pub fn dash_launch_gores(
    state: &DashGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:21419-21426` — the 20 jitter dusts.
    for _ in 0..20 {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(state.x, state.y),
            state.width, state.height, 31, 0.0, 0.0, 100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.position.x += rng.next_range(-5, 6) as f32;
        d.position.y += rng.next_range(-5, 6) as f32;
        d.velocity *= 0.2;
        d.scale *= 1.0 + rng.next_max(20) as f32 * 0.01;
    }
    // `:21427-21434` — the two gores. `(float)(base.width / 2)` and
    // `(float)(base.height / 2)` keep the C# INT division.
    let cx = state.x + (state.width / 2) as f32;
    let cy = state.y + (state.height / 2) as f32;
    for offset_y in [-34.0f32, -14.0] {
        let pos = Vec2::new(cx - 24.0, cy + offset_y);
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.velocity.x = rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y = rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity *= 0.4;
    }
}

/// The sandstorm hold snapshot (`DoubleJumpVisuals`, `:21905-21926`).
#[derive(Clone, Copy, Debug)]
pub struct SandstormGoreState {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub width: i32,
    pub height: i32,
    pub grav_dir: f32,
    /// `Player.jump` — the hold counter the `num4` scale reads.
    pub jump: i32,
    /// `Player.miscCounter` (`Player.cs:1207` port: u32) — the %3 gate.
    pub misc_counter: u32,
}

/// `Player.DoubleJumpVisuals`' sandstorm arm (`Player.cs:21905-21926`):
/// the 3 type-124 dusts then the `miscCounter % 3` gore (site 3),
/// type `Next(220,223)`, scale `num4 = (jump/75 + 1)/2`, the velocity
/// OVERWRITE `playerVel * 0.3 * num4`, alpha 100. The caller owns the
/// `isPerformingJump` gate and the `sandStorm = true` flag.
pub fn sandstorm_jump_gore(
    state: &SandstormGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:21907-21912` — the height pick (-6 reversed) and the scale.
    let num3 = if state.grav_dir == -1.0 { -6 } else { state.height };
    let num4 = (state.jump as f32 / 75.0 + 1.0) / 2.0;
    // `:21913-21918` — the 3 dusts.
    for _ in 0..3 {
        let pos = Vec2::new(state.x, state.y + (num3 / 2) as f32);
        let idx = dust.new_dust(
            rng, ctx, world, pos, state.width, 32, 124,
            state.vx * 0.3, state.vy * 0.3, 150, Color::default(), num4,
        );
        let d = dust.slot_mut(idx);
        d.velocity *= 0.5 * num4;
        d.fade_in = 1.5 * num4;
    }
    // `:21919` — `sandStorm = true` is the caller's flag write.
    // `:21920-21925` — the %3 gate (no draw).
    if state.misc_counter % 3 == 0 {
        let pos = Vec2::new(
            state.x + (state.width / 2) as f32 - 18.0,
            state.y + (num3 / 2) as f32,
        );
        let vel = Vec2::new(-state.vx, -state.vy);
        let ty = rng.next_range(220, 223);
        let idx = gore.new_gore(rng, world, pos, vel, ty, num4);
        let g = gore.slot_mut(idx);
        g.velocity = Vec2::new(state.vx, state.vy) * 0.3 * num4;
        g.alpha = 100;
    }
}

/// The hair-dye snapshot (`UpdateHairDyeDust`, `:24268-24294`).
#[derive(Clone, Copy, Debug)]
pub struct HairDyeGoreState {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub width: i32,
}

/// `Player.UpdateHairDyeDust`'s DUST arm (`Player.cs:24272-24282`):
/// the `Next(45)` gate ALWAYS draws; when it hits, one type
/// `Next(139,143)` dust + the six post rolls. Runs FIRST so the gore
/// arm's gate draw lands after it in C# order.
pub fn hair_dye_dust_arm(
    state: &HairDyeGoreState,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:24272`.
    if rng.next_max(45) == 0 {
        // `:24274` — the type draw is an argument, before the internals.
        let ty = rng.next_range(139, 143);
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(state.x, state.y),
            state.width, 8, ty, 0.0, 0.0, 0, Color::default(), 1.2,
        );
        // `:24275-24281` — the six rolls + the arithmetic tail.
        let d = dust.slot_mut(idx);
        d.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.x += rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y += rng.next_range(-50, 51) as f32 * 0.01;
        d.velocity.y -= 1.0;
        d.scale *= 0.7 + rng.next_range(-30, 31) as f32 * 0.01;
        d.velocity += Vec2::new(state.vx, state.vy) * 0.2;
    }
}

/// `Player.UpdateHairDyeDust`'s GORE arm (`Player.cs:24283-24293`,
/// site 4): the `Next(225)` gate ALWAYS draws; when it hits, type
/// `Next(276,283)` at `(x + Next(width), y + Next(8))`, the player
/// velocity, then the six post rolls.
pub fn hair_dye_gore_arm(
    state: &HairDyeGoreState,
    gore: &mut GorePool,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:24283`.
    if rng.next_max(225) == 0 {
        // `:24285` — the argument draws in C# order: Type first, then
        // Position.X's Next(width), then Position.Y's Next(8).
        let ty = rng.next_range(276, 283);
        let pos = Vec2::new(
            state.x + rng.next_max(state.width) as f32,
            state.y + rng.next_max(8) as f32,
        );
        let vel = Vec2::new(state.vx, state.vy);
        let idx = gore.new_gore(rng, world, pos, vel, ty, 1.0);
        // `:24286-24292` — the six rolls + the arithmetic tail.
        let g = gore.slot_mut(idx);
        g.velocity.x *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y *= 1.0 + rng.next_range(-50, 51) as f32 * 0.01;
        g.scale *= 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
        g.velocity.x += rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y += rng.next_range(-50, 51) as f32 * 0.01;
        g.velocity.y -= 1.0;
        g.velocity += Vec2::new(state.vx, state.vy) * 0.2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn air() -> World {
        World::new("t".into(), 200, 200)
    }

    fn ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::ZERO,
            screen_size: Vec2::new(1920.0, 1080.0),
            max_dust_to_draw: 6000,
        }
    }

    /// `Player.cs:21419-21434` — the dash tail: 20×(6 + 3) dust draws
    /// then 2×(type + 2 kicks + 2 overwrite rolls).
    #[test]
    fn w4_g17_the_dash_tail_draws_the_jitter_dusts_then_the_overwrites() {
        let w = air();
        let s = DashGoreState { x: 100.0, y: 200.0, width: 20, height: 42 };
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        dash_launch_gores(&s, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(7);
        for _ in 0..20 {
            let _px = twin.next_max(16); // width 20 - 4
            let _py = twin.next_max(38); // height 42 - 4
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
            let _jx = twin.next_range(-5, 6);
            let _jy = twin.next_range(-5, 6);
            let _sc = twin.next_max(20);
        }
        for _ in 0..2 {
            let _ty = twin.next_range(61, 64);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            let _ox = twin.next_range(-50, 51);
            let _oy = twin.next_range(-50, 51);
        }
        assert_eq!(rng.peek(), twin.peek(), "the dash tail mirrors the C# draws");
        // The anchors: centre (110,221) - (24,34)/(24,14).
        assert_eq!(gore.slot(0).position, Vec2::new(86.0, 187.0));
        assert_eq!(gore.slot(1).position, Vec2::new(86.0, 207.0));
        // The overwrite: |v| <= 0.5*0.4 = 0.2 per axis.
        for g in &gore.slots()[..2] {
            assert!(g.velocity.x.abs() <= 0.2 && g.velocity.y.abs() <= 0.2);
            assert!((61..=63).contains(&g.gore_type));
        }
    }

    /// `Player.cs:21913-21925` — the sandstorm arm: 3×6 dust draws,
    /// the %3 gate, then type + kicks; the velocity is OVERWRITTEN.
    #[test]
    fn w4_g17_the_sandstorm_gore_fires_on_the_mod_three_tick() {
        let w = air();
        let mut s = SandstormGoreState {
            x: 100.0, y: 200.0, vx: 2.0, vy: -4.0,
            width: 20, height: 42, grav_dir: 1.0, jump: 30, misc_counter: 6,
        };
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        sandstorm_jump_gore(&s, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(9);
        for _ in 0..3 {
            let _px = twin.next_max(16);
            let _py = twin.next_max(28);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        let _ty = twin.next_range(220, 223);
        let _ky = twin.next_range(10, 31);
        let _kx = twin.next_range(-20, 21);
        assert_eq!(rng.peek(), twin.peek(), "the sandstorm arm mirrors");
        assert!(gore.slot(0).active);
        assert!((220..=222).contains(&gore.slot(0).gore_type));
        // num4 = (30/75 + 1)/2 = 0.7; velocity = (2,-4)*0.3*0.7.
        let num4 = (30.0f32 / 75.0 + 1.0) / 2.0;
        assert_eq!(gore.slot(0).velocity, Vec2::new(2.0, -4.0) * 0.3 * num4);
        assert_eq!(gore.slot(0).alpha, 100);
        assert_eq!(gore.slot(0).scale, num4);
        // The anchor: x + 10 - 18, y + 42/2 = (92, 221).
        assert_eq!(gore.slot(0).position, Vec2::new(92.0, 221.0));
        // A non-%3 tick spawns no gore but still draws the dusts.
        s.misc_counter = 7;
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(9);
        sandstorm_jump_gore(&s, &mut gore2, &mut dust2, &ctx(), &mut rng2, &w);
        assert!(!gore2.slot(0).active);
    }

    /// `Player.cs:24272-24293` — both gates ALWAYS draw; the twin
    /// replays the gate draws and (when they hit) the arm rolls.
    #[test]
    fn w4_g17_the_hair_dye_gates_always_draw() {
        let w = air();
        let s = HairDyeGoreState { x: 100.0, y: 200.0, vx: 1.0, vy: -2.0, width: 20 };
        // Scan seeds until both gates hit, then verify the full order.
        let mut hit_seed = None;
        for seed in 0..500 {
            let mut probe = UnifiedRandom::new(seed);
            let dust_hit = probe.next_max(45) == 0;
            // The dust arm's draws when it hits (type + 6 internals +
            // 5 post rolls) shift the gore gate — replay them.
            if dust_hit {
                let _ty = probe.next_range(139, 143);
                let _px = probe.next_max(16);
                let _py = probe.next_max(4); // height 8 - 4
                let _vx = probe.next_range(-20, 21);
                let _vy = probe.next_range(-20, 21);
                let _row = probe.next_max(3);
                let _scale = probe.next_range(-20, 21);
                for _ in 0..4 {
                    let _r = probe.next_range(-50, 51);
                }
                let _sc = probe.next_range(-30, 31);
            }
            if dust_hit && probe.next_max(225) == 0 {
                hit_seed = Some(seed);
                break;
            }
        }
        let seed = hit_seed.expect("a both-gates seed exists in 0..500");
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(seed);
        hair_dye_dust_arm(&s, &mut dust, &ctx(), &mut rng, &w);
        hair_dye_gore_arm(&s, &mut gore, &mut rng, &w);
        assert!(gore.slot(0).active, "the gore gate hit");
        assert!((276..=282).contains(&gore.slot(0).gore_type));
        // The position stays inside the spawn box.
        let p = gore.slot(0).position;
        assert!(p.x >= 100.0 && p.x < 120.0 && p.y >= 200.0 && p.y < 208.0);
        // A gate-miss seed draws exactly the two gate rolls.
        let mut miss = None;
        for cand in 0..500 {
            let mut probe = UnifiedRandom::new(cand);
            if probe.next_max(45) != 0 && probe.next_max(225) != 0 {
                miss = Some(cand);
                break;
            }
        }
        let m = miss.expect("a both-miss seed exists");
        let mut gore2 = GorePool::new();
        let mut dust2 = DustPool::new();
        let mut rng2 = UnifiedRandom::new(m);
        hair_dye_dust_arm(&s, &mut dust2, &ctx(), &mut rng2, &w);
        hair_dye_gore_arm(&s, &mut gore2, &mut rng2, &w);
        let mut twin = UnifiedRandom::new(m);
        let _g1 = twin.next_max(45);
        let _g2 = twin.next_max(225);
        assert_eq!(rng2.peek(), twin.peek(), "two gate draws only");
        assert!(!gore2.slot(0).active);
    }
}
