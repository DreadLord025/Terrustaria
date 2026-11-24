//! Player-side `Gore.NewGore*` sites, wave A: the extra-jump visual
//! bursts (`Player.JumpMovement`, `Player.cs:20531-21032`). WIRED
//! (W4-g17 wiring wave): the `player.rs` extra-jump chain queues the
//! committed jump (`pending_jump_gores`) AFTER the velocity/jump
//! writes; `world_sim::player`'s tick-tail drain runs the burst on the
//! `UpdatePlayers` stream with the POST-jump snapshot — the
//! `npc_hit_effect_a.rs` (W4-d55) precedent.
//!
//! ## Scope — this commit's sites (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1-3 | `:20855`/`:20858`/`:20861` | the Fart jump's 3× Next(435,438) |
//! | 4 | `:20989` | the Santank jump's 3× Next(61,63) loop |
//! | 5-7 | `:21013`/`:21016`/`:21019` | the Cloud jump's 3× Next(11,14) |
//!
//! The dash (`:21427-21434`), sandstorm (`:21922`), hair-dye
//! (`:24285`) and dodge/death sites land in the follow-up commits.
//!
//! ## RNG contract (the player tick's `Main.rand` stream)
//!
//! The jump arms run AFTER the velocity/jump writes (`:20847-20848`
//! etc.) — the caller hands the POST-jump velocity in. Per arm:
//!
//! * Fart: 10 dusts first — per dust the `NewDust` internals (type
//!   188 hits no tweak: the 6 default-block draws, `Dust.cs:170-190`)
//!   then the two post-write scales (NO draws — the X/Y rewrites are
//!   arithmetic). Then per gore: `Next(435,438)` type (an argument,
//!   before the kicks), the kicks (2, pool-side), then the X/Y
//!   post-writes (no draws). 3 gores × 3 draws + 10 × 6 dust draws.
//! * Santank: 15 iterations × 2 dusts — the type-4 dust (no tweak, 6
//!   draws, the colour (250,230,230,150) is data) then the type-6
//!   dust (the `:214` tweak: +1 `Next(-10,6)`), the even-iteration
//!   `fadeIn = NextFloat * 2` (1). Then 3 gores: `Next(61,63)` type,
//!   kicks (2), the `NextVector2Circular(1,1)` velocity add (2: unit
//!   `NextFloat` + radius `NextFloat`, `Utils.cs:1261-1269`).
//! * Cloud: 10 type-16 dusts (no tweak, 6 each) + 3 gores of
//!   `Next(11,14)` (type + 2 kicks each).
//!
//! The post-spawn writes land on the returned slot, the sentinel
//! included (the `npc_hit_effect_a.rs` convention).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * The arm gates (`flag7`/`flag10`/the Cloud fall-through,
//!   `:20698-20748`'s priority chain), the `isPerformingJump_*`
//!   latch, the sound cue and the velocity/jump writes are the
//!   movement lane's — this module runs the visual tail of a
//!   COMMITTED jump.
//! * `gravDir` rides [`JumpGoreState`] (the port's Player carries
//!   only the `grav_control` flag, `player.rs:926`); the `num9`/
//!   `num20`/`num25` height picks (`height`, or 0 when gravDir == -1)
//!   are computed here exactly.
//! * `base.width`/`base.height` are C# INTs — the state carries them
//!   as i32 so `(float)(base.width / 2)` keeps the int division; the
//!   `Center`/`Bottom`/`Right` helpers use the float forms
//!   (`Entity.cs:50-60`/`:122-132`).
//! * netMode: single-player port, the pool's own convention.
//!
//! See docs/ANALYSIS.md "The player gore sites (W4-g17, wave A)".
// W4-g17 wiring: the jump tails are live (world_sim::player's drain);
// the module's own witnesses keep the pure-fn contracts pinned.

use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The player snapshot the jump visuals read — the POST-jump state
/// (the velocity/jump writes already ran, `:20847-20848`).
#[derive(Clone, Copy, Debug)]
pub struct JumpGoreState {
    /// `base.position` (`Entity.cs:18`).
    pub x: f32,
    pub y: f32,
    /// `base.velocity` — post-jump (`:20847`).
    pub vx: f32,
    pub vy: f32,
    /// `base.width` / `base.height` — C# ints (`Entity.cs:22-26`).
    pub width: i32,
    pub height: i32,
    /// `Player.gravDir` (`Player.cs:1188`).
    pub grav_dir: f32,
}

impl JumpGoreState {
    /// `Entity.Center` (`Entity.cs:50-60`) — float half-width.
    fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width as f32 / 2.0, self.y + self.height as f32 / 2.0)
    }

    /// `Entity.Bottom` (`Entity.cs:122-132`).
    fn bottom(&self) -> Vec2 {
        Vec2::new(self.x + self.width as f32 / 2.0, self.y + self.height as f32)
    }

    /// `Entity.Right` (`Entity.cs:104-114`) — only X is read here.
    fn right_x(&self) -> f32 {
        self.x + self.width as f32
    }

    /// The `num9`/`num20`/`num25` pick (`:20841-20845` etc.): the
    /// height, or 0 under reversed gravity.
    fn grav_height(&self) -> i32 {
        if self.grav_dir == -1.0 { 0 } else { self.height }
    }
}

/// `Utils.NextVector2Unit` (`Utils.cs:1261-1264`) — ONE `NextFloat`.
fn next_vector2_unit(rng: &mut UnifiedRandom) -> Vec2 {
    let angle = std::f32::consts::PI * 2.0 * rng.next_float();
    Vec2::new((angle as f64).cos() as f32, (angle as f64).sin() as f32)
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`).
fn next_vector2_circular(rng: &mut UnifiedRandom, half_w: f32, half_h: f32) -> Vec2 {
    let unit = next_vector2_unit(rng);
    let r = rng.next_float();
    Vec2::new(unit.x * half_w * r, unit.y * half_h * r)
}

/// The shared first-dust shape of the Fart/Santank/Cloud arms
/// (`:20851-20853` / `:20962-20965` / `:21009-21011`): a 102×32 box
/// at `(x-34, y+gravHeight-16)`, speed `(-vx*0.5, vy*0.5)`, alpha
/// 100, scale 1.5, then the X/Y half-and-drift rewrite.
fn jump_puff_dust(
    state: &JumpGoreState,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
    dust_type: i32,
    color: Color,
) -> usize {
    let pos = Vec2::new(state.x - 34.0, state.y + state.grav_height() as f32 - 16.0);
    let idx = dust.new_dust(
        rng, ctx, world, pos, 102, 32, dust_type,
        -state.vx * 0.5, state.vy * 0.5, 100, color, 1.5,
    );
    let d = dust.slot_mut(idx);
    d.velocity.x = d.velocity.x * 0.5 - state.vx * 0.1;
    d.velocity.y = d.velocity.y * 0.5 - state.vy * 0.3;
    idx
}

/// The Fart/Cloud gore post-write (`:20856-20857` / `:21014-21015`):
/// `velocity.X = velocity.X * 0.1 - playerVX * 0.1`, Y the 0.05 twin.
fn fart_gore_post(gore: &mut GorePool, idx: usize, state: &JumpGoreState) {
    let g = gore.slot_mut(idx);
    g.velocity.x = g.velocity.x * 0.1 - state.vx * 0.1;
    g.velocity.y = g.velocity.y * 0.1 - state.vy * 0.05;
}

/// `Player.JumpMovement`'s Fart arm (`Player.cs:20838-20864`): the
/// 10 type-188 puffs then the 3 gores (sites 1-3) at the centre-left
/// / far-left / far-right anchors, types `Next(435,438)`.
pub fn fart_jump_gores(
    state: &JumpGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:20849-20854` — the 10 puffs.
    for _ in 0..10 {
        jump_puff_dust(state, dust, ctx, rng, world, 188, Color::default());
    }
    // `:20855-20863` — the three gores. `(float)(base.width / 2)` is
    // INT division; the velocity argument is the negated player vel.
    let gh = state.grav_height() as f32;
    let vel = Vec2::new(-state.vx, -state.vy);
    let anchors = [
        Vec2::new(state.x + (state.width / 2) as f32 - 16.0, state.y + gh - 16.0),
        Vec2::new(state.x - 36.0, state.y + gh - 16.0),
        Vec2::new(state.x + state.width as f32 + 4.0, state.y + gh - 16.0),
    ];
    for pos in anchors {
        let ty = rng.next_range(435, 438);
        let idx = gore.new_gore(rng, world, pos, vel, ty, 1.0);
        fart_gore_post(gore, idx, state);
    }
}

/// `Player.JumpMovement`'s Santank arm (`Player.cs:20949-20995`): the
/// 15×2 dust pairs then the 3-gore loop (site 4), types `Next(61,63)`,
/// the `Bottom.Y - 22` anchor row and the Circular(1,1) velocity add.
pub fn santank_jump_gores(
    state: &JumpGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:20960-20972` — the 15 iterations.
    for i in 0..15 {
        // `:20962-20965` — the type-4 puff, colour (250,230,230,150).
        let idx = jump_puff_dust(
            state, dust, ctx, rng, world, 4, Color::new(250, 230, 230, 150),
        );
        dust.slot_mut(idx).no_gravity = true;
        // `:20966-20971` — the type-6 puff (alpha 20), the Y decrement
        // and the even-iteration fadeIn roll.
        let pos = Vec2::new(state.x - 34.0, state.y + state.grav_height() as f32 - 16.0);
        let idx2 = dust.new_dust(
            rng, ctx, world, pos, 102, 32, 6,
            -state.vx * 0.5, state.vy * 0.5, 20, Color::default(), 1.5,
        );
        let d = dust.slot_mut(idx2);
        d.velocity.y -= 1.0;
        if i % 2 == 0 {
            d.fade_in = rng.next_float() * 2.0;
        }
    }
    // `:20973-20994` — the gore row. NOTE: `y = Bottom.Y - 22` reads
    // the FULL height even under reversed gravity (the C# does not
    // use num20 here).
    let y = state.bottom().y - 22.0;
    let vel = Vec2::new(-state.vx, -state.vy);
    for case in 0..3 {
        // `:20976-20988` — the switch anchors.
        let pos = match case {
            0 => Vec2::new(state.center().x - 16.0, y),
            1 => Vec2::new(state.x - 36.0, y),
            _ => Vec2::new(state.right_x() + 4.0, y),
        };
        let ty = rng.next_range(61, 63);
        let idx = gore.new_gore(rng, world, pos, vel, ty, 1.0);
        // `:20990-20993` — the sentinel slot included.
        let g = gore.slot_mut(idx);
        g.velocity *= 0.1;
        g.velocity.x -= state.vx * 0.1;
        g.velocity.y -= state.vy * 0.05;
        g.velocity += next_vector2_circular(rng, 1.0, 1.0) * 0.5;
    }
}

/// `Player.JumpMovement`'s Cloud fall-through (`Player.cs:20996-21022`):
/// the 10 type-16 puffs then the 3 gores (sites 5-7), types
/// `Next(11,14)` — the Fart shape with the cloud type row.
pub fn cloud_jump_gores(
    state: &JumpGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:21007-21012`.
    for _ in 0..10 {
        jump_puff_dust(state, dust, ctx, rng, world, 16, Color::default());
    }
    // `:21013-21021`.
    let gh = state.grav_height() as f32;
    let vel = Vec2::new(-state.vx, -state.vy);
    let anchors = [
        Vec2::new(state.x + (state.width / 2) as f32 - 16.0, state.y + gh - 16.0),
        Vec2::new(state.x - 36.0, state.y + gh - 16.0),
        Vec2::new(state.x + state.width as f32 + 4.0, state.y + gh - 16.0),
    ];
    for pos in anchors {
        let ty = rng.next_range(11, 14);
        let idx = gore.new_gore(rng, world, pos, vel, ty, 1.0);
        fart_gore_post(gore, idx, state);
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

    fn state() -> JumpGoreState {
        JumpGoreState {
            x: 100.0, y: 200.0, vx: 2.0, vy: -5.01,
            width: 20, height: 42, grav_dir: 1.0,
        }
    }

    /// `Player.cs:20849-20863` — the Fart arm: 10×6 dust draws then
    /// 3×(type + 2 kicks); the twin replays the exact order.
    #[test]
    fn w4_g17_the_fart_jump_draws_ten_puffs_then_three_gores() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        fart_jump_gores(&state(), &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(7);
        for _ in 0..10 {
            // NewDust default block (type 188: no tweak).
            let _px = twin.next_max(98); // width 102 - 4
            let _py = twin.next_max(28); // height 32 - 4
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..3 {
            let _ty = twin.next_range(435, 438);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the fart arm mirrors the C# draws");
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert!(types.iter().all(|&t| (435..=437).contains(&t)));
        // The anchors: centre-16 / far-left / far-right at y+42-16.
        assert_eq!(gore.slot(0).position, Vec2::new(94.0, 226.0));
        assert_eq!(gore.slot(1).position, Vec2::new(64.0, 226.0));
        assert_eq!(gore.slot(2).position, Vec2::new(124.0, 226.0));
    }

    /// `Player.cs:20841-20845` — reversed gravity zeroes the height
    /// pick: the gores sit at y-16, not y+26.
    #[test]
    fn w4_g17_the_reversed_gravity_zeroes_the_height_pick() {
        let w = air();
        let mut s = state();
        s.grav_dir = -1.0;
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        fart_jump_gores(&s, &mut gore, &mut dust, &ctx(), &mut rng, &w);
        assert_eq!(gore.slot(0).position, Vec2::new(94.0, 184.0));
    }

    /// `Player.cs:20960-20994` — the Santank arm: 15×(6 + 7 + even
    /// fadeIn) dust draws then 3×(type + kicks + circular). The type-6
    /// tweak adds Next(-10,6); the even iterations add one NextFloat.
    #[test]
    fn w4_g17_the_santank_jump_draws_the_puff_pairs_then_the_circular_gores() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        santank_jump_gores(&state(), &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(9);
        let dust_block = |r: &mut UnifiedRandom| {
            let _px = r.next_max(98);
            let _py = r.next_max(28);
            let _vx = r.next_range(-20, 21);
            let _vy = r.next_range(-20, 21);
            let _row = r.next_max(3);
            let _scale = r.next_range(-20, 21);
        };
        for i in 0..15 {
            dust_block(&mut twin); // type 4: no tweak
            dust_block(&mut twin); // type 6 default block
            let _tweak = twin.next_range(-10, 6); // the :214 Y rewrite
            if i % 2 == 0 {
                let _fade = twin.next_float();
            }
        }
        for _ in 0..3 {
            let _ty = twin.next_range(61, 63);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            let _unit = twin.next_float(); // Circular unit
            let _rad = twin.next_float(); // Circular radius
        }
        assert_eq!(rng.peek(), twin.peek(), "the santank arm mirrors the C# draws");
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert!(types.iter().all(|&t| (61..=62).contains(&t)));
        // y = Bottom.Y - 22 = 200 + 42 - 22 = 220.
        assert_eq!(gore.slot(0).position, Vec2::new(94.0, 220.0));
        assert_eq!(gore.slot(1).position, Vec2::new(64.0, 220.0));
        assert_eq!(gore.slot(2).position, Vec2::new(124.0, 220.0));
    }

    /// `Player.cs:21007-21021` — the Cloud arm: the Fart shape with
    /// type-16 puffs and Next(11,14) gores.
    #[test]
    fn w4_g17_the_cloud_jump_mirrors_the_fart_shape() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(11);
        cloud_jump_gores(&state(), &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(11);
        for _ in 0..10 {
            let _px = twin.next_max(98);
            let _py = twin.next_max(28);
            let _vx = twin.next_range(-20, 21);
            let _vy = twin.next_range(-20, 21);
            let _row = twin.next_max(3);
            let _scale = twin.next_range(-20, 21);
        }
        for _ in 0..3 {
            let _ty = twin.next_range(11, 14);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
        }
        assert_eq!(rng.peek(), twin.peek(), "the cloud arm mirrors the C# draws");
        let types: Vec<i32> = gore.slots()[..3].iter().map(|g| g.gore_type).collect();
        assert!(types.iter().all(|&t| (11..=13).contains(&t)));
        // The post-write: X = kick*0.1 - vx*0.1 with vx = 2.
        for g in &gore.slots()[..3] {
            assert!(g.velocity.x.abs() < 1.0);
        }
    }
}
