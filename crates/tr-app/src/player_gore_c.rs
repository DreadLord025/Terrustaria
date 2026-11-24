//! Player-side `Gore.NewGore*` sites, wave C: the dodge bursts —
//! `DoMysticSashDodge` + `NinjaDodge` (`Player.cs:38057-38165`).
//! WIRED (W4-g17 wiring wave): the `player.rs` dodge handlers queue
//! `pending_dodge_gores` (the immunity write stays inline, the C#
//! order); `world_sim::player`'s drain runs the burst on the
//! `UpdatePlayers` stream — the dodge roll already drew from it, the
//! burst follows (the `npc_hit_effect_a.rs` precedent).
//!
//! ## Scope — this commit's sites (source order)
//!
//! | # | C# site | arm |
//! | --- | --- | --- |
//! | 1-5 | `:38083`-`:38107` | Mystic Sash — 5× Next(11,14), alpha 100 |
//! | 6-10 | `:38136`-`:38156` | Ninja — 5× Next(61,64), no alpha |
//!
//! Both bursts share the 5-gore velocity fan: centre, (+1.5,+1.5),
//! (-1.5,+1.5), (+1.5,-1.5), (-1.5,-1.5) — each axis carries its own
//! `Next(-50,51)*0.01` roll, then the whole velocity ×0.4.
//!
//! ## RNG contract (the player tick's `Main.rand` stream)
//!
//! * Mystic Sash: 50 dust iterations FIRST — per iteration the
//!   `NewDustDirect` scale argument `Next(10,26)*0.1` (1), the
//!   internals (type 267 hits no tweak: 6 draws), the fadeIn
//!   `NextFloat` (1), the scale `NextFloat` (1), the colour-lerp
//!   `NextFloat` (1), then the `CloneDust` — a fresh `NewDust` of the
//!   same type at 0×0/scale 1 (6 more draws; the sentinel early
//!   return, `Dust.cs:265-268`, is the pool's). 16 draws × 50. Then
//!   per gore: `Next(11,14)` type, kicks (2), the X/Y fan rolls (2),
//!   scale = 1.5 / alpha = 100 / ×0.4 (no draws).
//! * Ninja: 100 type-31 dusts first — per dust the internals (6),
//!   the position jitter `Next(-20,21)` ×2, the scale `Next(40)`
//!   (1), the `Next(2)` gate (1) and, when it hits, a second
//!   `Next(40)` scale roll (1). Then per gore: `Next(61,64)` type,
//!   kicks (2), the fan rolls (2); scale = 1.5, ×0.4, NO alpha write.
//!
//! The post-spawn writes land on the returned slot, the sentinel
//! included (the `npc_hit_effect_a.rs` convention).
//!
//! ## Documented drops / stand-ins (this wave)
//!
//! * The immunity grant (`SetImmuneTimeForAllTypes(longInvince ?
//!   120 : 80)`), the `brainOfConfusionDodgeAnimationCounter`, the
//!   `AddBuff(321, 240)` and the `NetMessage.SendData(62, …)` dodge
//!   syncs (`:38048-38054`, `:38113-38116`, `:38121`, `:38161-38164`)
//!   are the caller's / the net lane's — this module runs the visual
//!   burst only.
//! * The dust `shader` writes (`GameShaders.Armor.GetSecondaryShader`,
//!   `:38129`) are dropped fields (the pool's convention).
//! * `Color.Lerp` (`:38070`) is XNA's per-channel `MathHelper.Lerp`
//!   with the ctor's truncating cast — [`lerp_color`] mirrors the
//!   `dust_table.rs` helper (private there).
//!
//! See docs/ANALYSIS.md "The player gore sites (W4-g17, wave C)".
// W4-g17 wiring: both dodge bursts are live (world_sim::player's drain).

use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::{UnifiedRandom, World};

/// The player snapshot the dodge bursts read.
#[derive(Clone, Copy, Debug)]
pub struct DodgeGoreState {
    pub x: f32,
    pub y: f32,
    /// C# int hitbox (`Entity.cs:22-26`).
    pub width: i32,
    pub height: i32,
}

impl DodgeGoreState {
    /// The shared gore anchor: centre - (24, 24) with INT division
    /// (`:38083` / `:38136`).
    fn anchor(&self) -> Vec2 {
        Vec2::new(
            self.x + (self.width / 2) as f32 - 24.0,
            self.y + (self.height / 2) as f32 - 24.0,
        )
    }
}

/// XNA `Color.Lerp` — per-channel `MathHelper.Lerp`, truncating cast
/// (the `dust_table.rs::lerp_color` twin; the channels here stay in
/// range so the clamp never fires).
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| {
        let v = (x as f32 + (y as f32 - x as f32) * t) as i32;
        v.clamp(0, 255) as u8
    };
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// The 5-gore velocity fan (`:38085-38111` / `:38138-38160`): the
/// per-axis offsets `(0,0)`, `(+1.5,+1.5)`, `(-1.5,+1.5)`,
/// `(+1.5,-1.5)`, `(-1.5,-1.5)` — the negative arms SUBTRACT the roll
/// (`-1.5 - roll`), the positive arms ADD it. Each axis rolls its own
/// `Next(-50,51)`; the kicks are OVERWRITTEN, then ×0.4.
const FAN: [(f32, f32); 5] = [
    (0.0, 0.0),
    (1.5, 1.5),
    (-1.5, 1.5),
    (1.5, -1.5),
    (-1.5, -1.5),
];

fn fan_gore_velocity(g: &mut tr_core::gore::Gore, rng: &mut UnifiedRandom, ox: f32, oy: f32) {
    // The sign rides the offset: `0 + roll`, `1.5 + roll`, `-1.5 - roll`.
    let sign_x = if ox < 0.0 { -1.0 } else { 1.0 };
    let sign_y = if oy < 0.0 { -1.0 } else { 1.0 };
    g.velocity.x = ox + sign_x * rng.next_range(-50, 51) as f32 * 0.01;
    g.velocity.y = oy + sign_y * rng.next_range(-50, 51) as f32 * 0.01;
    g.velocity *= 0.4;
}

/// `Player.DoMysticSashDodge`'s visual burst (`Player.cs:38060-38112`):
/// the 50 type-267 dust + clone pairs then the 5 gores (sites 1-5),
/// types `Next(11,14)`, scale 1.5, alpha 100.
pub fn mystic_sash_dodge_gores(
    state: &DodgeGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:38060-38079` — the 50 dust + clone iterations.
    for _ in 0..50 {
        // `:38062` — the scale argument draw rides BEFORE the internals.
        let scale = rng.next_range(10, 26) as f32 * 0.1;
        let idx = dust.new_dust_direct(
            rng, ctx, world, Vec2::new(state.x, state.y),
            state.width, state.height, 267, 0.0, 0.0, 255, Color::default(), scale,
        );
        // `:38063-38072` — the post-writes (3 NextFloat draws).
        let d = dust.slot_mut(idx);
        d.no_light = true;
        d.no_gravity = true;
        d.velocity *= 2.5;
        d.velocity.y -= 0.5;
        d.velocity.y -= 1.0;
        d.fade_in = 0.7 + 1.3 * rng.next_float();
        d.scale = 0.7 + 1.3 * rng.next_float();
        d.color = lerp_color(
            Color::new(90, 0, 255, 255),
            Color::new(40, 0, 255, 255),
            rng.next_float(),
        );
        d.alpha = 200;
        d.position.y += 5.0;
        // `:38073-38078` — the clone (its own NewDust draws) + copies.
        let idx2 = dust.clone_dust(rng, ctx, world, idx);
        let d2 = dust.slot_mut(idx2);
        d2.no_light = true;
        d2.no_gravity = true;
        d2.scale *= 0.35;
        d2.fade_in *= 0.35;
        d2.color = Color::WHITE;
    }
    // `:38080-38112` — the 5-gore fan.
    let pos = state.anchor();
    for (ox, oy) in FAN {
        let ty = rng.next_range(11, 14);
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.scale = 1.5;
        fan_gore_velocity(g, rng, ox, oy);
        g.alpha = 100;
    }
}

/// `Player.NinjaDodge`'s visual burst (`Player.cs:38122-38160`): the
/// 100 type-31 jitter dusts then the 5 gores (sites 6-10), types
/// `Next(61,64)`, scale 1.5, NO alpha write.
pub fn ninja_dodge_gores(
    state: &DodgeGoreState,
    gore: &mut GorePool,
    dust: &mut DustPool,
    ctx: &SpawnCtx,
    rng: &mut UnifiedRandom,
    world: &World,
) {
    // `:38122-38135` — the 100 jitter dusts.
    for _ in 0..100 {
        let idx = dust.new_dust(
            rng, ctx, world, Vec2::new(state.x, state.y),
            state.width, state.height, 31, 0.0, 0.0, 100, Color::default(), 2.0,
        );
        let d = dust.slot_mut(idx);
        d.position.x += rng.next_range(-20, 21) as f32;
        d.position.y += rng.next_range(-20, 21) as f32;
        d.velocity *= 0.4;
        d.scale *= 1.0 + rng.next_max(40) as f32 * 0.01;
        // `:38130-38134` — the Next(2) gate ALWAYS draws.
        if rng.next_max(2) == 0 {
            d.scale *= 1.0 + rng.next_max(40) as f32 * 0.01;
            d.no_gravity = true;
        }
    }
    // `:38136-38160` — the 5-gore fan (no alpha).
    let pos = state.anchor();
    for (ox, oy) in FAN {
        let ty = rng.next_range(61, 64);
        let idx = gore.new_gore(rng, world, pos, Vec2::ZERO, ty, 1.0);
        let g = gore.slot_mut(idx);
        g.scale = 1.5;
        fan_gore_velocity(g, rng, ox, oy);
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

    fn state() -> DodgeGoreState {
        DodgeGoreState { x: 100.0, y: 200.0, width: 20, height: 42 }
    }

    /// The dust default block (type 267/31 hit no tweak): 6 draws.
    fn dust_block(r: &mut UnifiedRandom, w: i32, h: i32) {
        let _px = r.next_max(w - 4);
        let _py = r.next_max(h - 4);
        let _vx = r.next_range(-20, 21);
        let _vy = r.next_range(-20, 21);
        let _row = r.next_max(3);
        let _scale = r.next_range(-20, 21);
    }

    /// `Player.cs:38060-38112` — the Mystic Sash burst: 50×16 dust
    /// draws then 5×(type + kicks + 2 fan rolls).
    #[test]
    fn w4_g17_the_mystic_sash_burst_draws_the_clone_pairs_then_the_fan() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(7);
        mystic_sash_dodge_gores(&state(), &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(7);
        for _ in 0..50 {
            let _sc = twin.next_range(10, 26); // the scale argument
            dust_block(&mut twin, 20, 42); // the direct internals
            let _fade = twin.next_float();
            let _scale = twin.next_float();
            let _lerp = twin.next_float();
            dust_block(&mut twin, 5, 5); // the clone's 0x0 → 5x5 box
        }
        for _ in 0..5 {
            let _ty = twin.next_range(11, 14);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            let _fx = twin.next_range(-50, 51);
            let _fy = twin.next_range(-50, 51);
        }
        assert_eq!(rng.peek(), twin.peek(), "the sash burst mirrors the C# draws");
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert!(types.iter().all(|&t| (11..=13).contains(&t)));
        for g in &gore.slots()[..5] {
            assert_eq!(g.scale, 1.5);
            assert_eq!(g.alpha, 100);
            assert_eq!(g.position, Vec2::new(86.0, 197.0));
        }
        // The fan: gore 1 drifts positive, gore 4 negative on both axes.
        assert!(gore.slot(1).velocity.x > 0.0 && gore.slot(1).velocity.y > 0.0);
        assert!(gore.slot(4).velocity.x < 0.0 && gore.slot(4).velocity.y < 0.0);
    }

    /// `Player.cs:38122-38160` — the Ninja burst: 100×(6 + 2 jitter +
    /// scale + gate [+ second scale]) dust draws then the fan.
    #[test]
    fn w4_g17_the_ninja_burst_draws_the_jitter_gate_then_the_fan() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(9);
        ninja_dodge_gores(&state(), &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let mut twin = UnifiedRandom::new(9);
        for _ in 0..100 {
            dust_block(&mut twin, 20, 42);
            let _jx = twin.next_range(-20, 21);
            let _jy = twin.next_range(-20, 21);
            let _sc = twin.next_max(40);
            if twin.next_max(2) == 0 {
                let _sc2 = twin.next_max(40);
            }
        }
        for _ in 0..5 {
            let _ty = twin.next_range(61, 64);
            let _ky = twin.next_range(10, 31);
            let _kx = twin.next_range(-20, 21);
            let _fx = twin.next_range(-50, 51);
            let _fy = twin.next_range(-50, 51);
        }
        assert_eq!(rng.peek(), twin.peek(), "the ninja burst mirrors the C# draws");
        let types: Vec<i32> = gore.slots()[..5].iter().map(|g| g.gore_type).collect();
        assert!(types.iter().all(|&t| (61..=63).contains(&t)));
        for g in &gore.slots()[..5] {
            assert_eq!(g.scale, 1.5);
            assert_eq!(g.alpha, 0, "the ninja fan writes no alpha");
        }
        // The fan's mixed signs: gore 2 (-X, +Y), gore 3 (+X, -Y).
        assert!(gore.slot(2).velocity.x < 0.0 && gore.slot(2).velocity.y > 0.0);
        assert!(gore.slot(3).velocity.x > 0.0 && gore.slot(3).velocity.y < 0.0);
    }

    /// The fan's centre gore keeps the pure roll (no ±1.5 offset) and
    /// the ×0.4 bound holds on every axis.
    #[test]
    fn w4_g17_the_fan_bounds_hold() {
        let w = air();
        let mut gore = GorePool::new();
        let mut dust = DustPool::new();
        let mut rng = UnifiedRandom::new(3);
        ninja_dodge_gores(&state(), &mut gore, &mut dust, &ctx(), &mut rng, &w);
        let c = gore.slot(0).velocity;
        assert!(c.x.abs() <= 0.5 * 0.4 && c.y.abs() <= 0.5 * 0.4);
        for g in &gore.slots()[1..5] {
            assert!(g.velocity.x.abs() <= (1.5 + 0.5) * 0.4);
            assert!(g.velocity.y.abs() <= (1.5 + 0.5) * 0.4);
        }
    }
}
