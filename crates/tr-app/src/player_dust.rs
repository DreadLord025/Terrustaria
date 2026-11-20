//! Player-tick `Dust.NewDust` sites — the drain half of the queue convention
//! (`Player.StickyMovement`'s `StickyDust`, `recall_teleport::new_dust`). The
//! player tick only knows WHERE a burst goes and snapshots the box/velocity at
//! the C# moment; this module replays each `Dust.NewDust` on the shared
//! `UpdatePlayers` stream (`Main.SwapRandom`, `Main.cs:17755-17757`) in exact
//! C# draw order, then applies the per-site post-arms. One draw = one stream
//! step; no new streams (S1-A-10). The pool is the App's `particles::DustPool`
//! (`Main.dust`), so the pressure bands / default block / ChildSafety / per-type
//! tweaks all ride `DustPool::new_dust` verbatim (`Dust.cs:75-256`).
//!
//! W4-d51 scope: the extra-jump chain (`Player.JumpMovement`,
//! `Player.cs:20818-21012`). The dash / fast-run / regen lanes land in later
//! batches of the same task. The `Gore.NewGore` siblings of the Fart / Santank /
//! Cloud arms (`:20855-20863`, `:20989-20993`, `:21013-21020`) are the gore
//! lane's (W4-d55) — their `Main.rand` draws are a documented skip here, exactly
//! as the rest of the port treats unported gore (docs/ANALYSIS.md).
//!
//! `gravDir` is the vanilla `1f` (the port has no gravity flip, the
//! `sticky_movement` convention, `Player.cs:1956`), so the `gravDir == -1`
//! height/offset folds collapse to their `+1` arms.

use crate::app::App;
use crate::player::{ExtraJump, JumpDust};
use glam::Vec2;
use tr_core::particles::Color;

/// One `Dust.NewDust` on the `UpdatePlayers` stream, returning the slot index
/// (the sentinel included — the C# writes `Main.dust[6000]` on a refusal).
fn nd(
    app: &mut App,
    pos: Vec2,
    w: i32,
    h: i32,
    ty: i32,
    sx: f32,
    sy: f32,
    alpha: i32,
    color: Color,
    scale: f32,
) -> usize {
    let ctx = app.spawn_ctx();
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    app.dust
        .new_dust(rng, &ctx, &app.world, pos, w, h, ty, sx, sy, alpha, color, scale)
}

/// Drain the extra-jump chain's dust burst (`Player.JumpMovement`,
/// `Player.cs:20818-21012`). Called right after the chain's `PlaySound(16)` /
/// Item-16 drain so the sound-then-dust order matches the source. Sandstorm /
/// Blizzard play no dust (sound only). The snapshot in [`JumpDust`] is taken
/// AFTER the ascent-velocity write, exactly as the C# reads `base.velocity`.
pub(crate) fn drain_jump_dust(app: &mut App, b: JumpDust) {
    match b.kind {
        // `:20787-20806`: Sandstorm / Blizzard — sound only, no dust.
        ExtraJump::Sandstorm | ExtraJump::Blizzard => {}
        ExtraJump::Sail => sail(app, &b),
        ExtraJump::Fart => fart(app, &b),
        ExtraJump::Unicorn => ring(app, &b, RingTy::Unicorn),
        ExtraJump::WallOfFleshGoat => ring(app, &b, RingTy::Goat),
        ExtraJump::Basilisk => ring(app, &b, RingTy::Basilisk),
        ExtraJump::Santank => santank(app, &b),
        ExtraJump::Cloud => cloud(app, &b),
    }
}

/// `:20818-20836` — the Sail's 30 type-253 puffs. Per dust: the six default
/// draws, then `velocity.X ± Next(30,71)*0.1` (even/odd), `velocity.Y +=
/// Next(-10,31)*0.1`, `noGravity`, `scale += Next(-10,41)*0.01`,
/// `velocity *= scale*0.7`, then the DEAD `vector` draws `Next(-100,101)`×2 +
/// `Next(81)` (the C# normalises/scales a local it never reads — the draws still
/// ride the stream).
fn sail(app: &mut App, b: &JumpDust) {
    let pos = Vec2::new(b.x, b.y + b.height);
    for l in 0..30 {
        let idx = nd(
            app,
            pos,
            b.width as i32,
            12,
            253,
            b.vx * 0.3,
            b.vy * 0.3,
            100,
            Color::default(),
            1.5,
        );
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        if l % 2 == 0 {
            d.velocity.x += rng.next_range(30, 71) as f32 * 0.1;
        } else {
            d.velocity.x -= rng.next_range(30, 71) as f32 * 0.1;
        }
        d.velocity.y += rng.next_range(-10, 31) as f32 * 0.1;
        d.no_gravity = true;
        d.scale += rng.next_range(-10, 41) as f32 * 0.01;
        d.velocity *= d.scale * 0.7;
        // The dead `vector` local (`:20833-20835`): three draws, no effect.
        let _ = rng.next_range(-100, 101);
        let _ = rng.next_range(-100, 101);
        let _ = rng.next_max(81);
    }
}

/// `:20849-20854` — the Fart's 10 type-188 puffs, box 102×32 at
/// `(x-34, y+height-16)`, speed `(-vx*0.5, vy*0.5)`. Post: `velocity.X =
/// vX*0.5 - vx*0.1`, `velocity.Y = vY*0.5 - vy*0.3` (no RNG). The three
/// `Gore.NewGore` siblings (`:20855-20863`) are the gore lane's (skip).
fn fart(app: &mut App, b: &JumpDust) {
    let pos = Vec2::new(b.x - 34.0, b.y + b.height - 16.0);
    for _ in 0..10 {
        let idx = nd(
            app,
            pos,
            102,
            32,
            188,
            -b.vx * 0.5,
            b.vy * 0.5,
            100,
            Color::default(),
            1.5,
        );
        let d = app.dust.slot_mut(idx);
        d.velocity.x = d.velocity.x * 0.5 - b.vx * 0.1;
        d.velocity.y = d.velocity.y * 0.5 - b.vy * 0.3;
    }
}

/// The ring arms' dust type: Unicorn rolls `SelectRandom(176,177,179)` (a
/// `Next(3)` per dust), Goat is a fixed 6, Basilisk a fixed 31.
#[derive(Clone, Copy)]
enum RingTy {
    Unicorn,
    Goat,
    Basilisk,
}

/// `:20874-20891` (Unicorn) / `:20902-20919` (Goat) / `:20930-20947`
/// (Basilisk) — the 5×14 mount ring. One `NextFloat` for the ring phase
/// (`num12 = 2π*NextFloat`) BEFORE the loop, then per dust: the type roll
/// (Unicorn only), the six default draws, then `scale = 1 + NextFloat*0.8` and
/// `fadeIn = NextFloat*2`. Position/velocity are the deterministic rotated
/// ring (`UnitY.RotatedBy(num13*2π/14 + phase) * 0.2*n`), no RNG.
fn ring(app: &mut App, b: &JumpDust, ty: RingTy) {
    let center = Vec2::new(b.cx, b.cy);
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let phase = std::f32::consts::PI * 2.0 * rng.next_float();
    for n in 0..5 {
        for num13 in 0..14 {
            let angle = (num13 as f32) * (std::f32::consts::PI * 2.0) / 14.0 + phase;
            let mut v3 = crate::bow_launch::rotated_by(Vec2::new(0.0, 1.0), angle as f64);
            v3 *= 0.2 * n as f32;
            let dust_ty = match ty {
                // `Utils.SelectRandom(Main.rand, 176, 177, 179)` — `Next(3)`.
                RingTy::Unicorn => {
                    let r = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
                    [176, 177, 179][r.next_max(3) as usize]
                }
                RingTy::Goat => 6,
                RingTy::Basilisk => 31,
            };
            let idx = nd(app, center, 0, 0, dust_ty, 0.0, 0.0, 0, Color::default(), 1.0);
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            let d = app.dust.slot_mut(idx);
            d.position = center + v3 * Vec2::new(50.0, 20.0);
            d.velocity = v3 + Vec2::new(0.0, 4.0); // gravDir 1
            d.no_gravity = true;
            d.scale = 1.0 + rng.next_float() * 0.8;
            d.fade_in = rng.next_float() * 2.0;
        }
    }
}

/// `:20960-20972` — the Santank's 15 pairs. Per iteration: a type-4 puff
/// (color 250,230,230,150, alpha 100, scale 1.5) with the Fart-style velocity
/// post-arms + `noGravity`, then a type-6 puff (alpha 20, scale 1.5) with
/// `velocity.Y--` and, on even iterations, `fadeIn = NextFloat*2`. The three
/// `Gore.NewGore` siblings (`:20989-20993`) are the gore lane's (skip).
fn santank(app: &mut App, b: &JumpDust) {
    let pos = Vec2::new(b.x - 34.0, b.y + b.height - 16.0);
    for num21 in 0..15 {
        let idx = nd(
            app,
            pos,
            102,
            32,
            4,
            -b.vx * 0.5,
            b.vy * 0.5,
            100,
            Color::new(250, 230, 230, 150),
            1.5,
        );
        let d = app.dust.slot_mut(idx);
        d.velocity.x = d.velocity.x * 0.5 - b.vx * 0.1;
        d.velocity.y = d.velocity.y * 0.5 - b.vy * 0.3;
        d.no_gravity = true;
        let idx = nd(
            app,
            pos,
            102,
            32,
            6,
            -b.vx * 0.5,
            b.vy * 0.5,
            20,
            Color::default(),
            1.5,
        );
        app.dust.slot_mut(idx).velocity.y -= 1.0;
        if num21 % 2 == 0 {
            let fade = app
                .rngs
                .stream(crate::app::RngKey::UpdatePlayers)
                .next_float()
                * 2.0;
            app.dust.slot_mut(idx).fade_in = fade;
        }
    }
}

/// `:21007-21012` — the Cloud's 10 type-16 puffs, box 102×32 at
/// `(x-34, y+height-16)`, speed `(-vx*0.5, vy*0.5)`, the Fart-style velocity
/// post-arms (no RNG). The two `Gore.NewGore` siblings (`:21013-21020`) are the
/// gore lane's (skip).
fn cloud(app: &mut App, b: &JumpDust) {
    let pos = Vec2::new(b.x - 34.0, b.y + b.height - 16.0);
    for _ in 0..10 {
        let idx = nd(
            app,
            pos,
            102,
            32,
            16,
            -b.vx * 0.5,
            b.vy * 0.5,
            100,
            Color::default(),
            1.5,
        );
        let d = app.dust.slot_mut(idx);
        d.velocity.x = d.velocity.x * 0.5 - b.vx * 0.1;
        d.velocity.y = d.velocity.y * 0.5 - b.vy * 0.3;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use tr_core::{UnifiedRandom, World};

    /// A drain rig: an empty world, the dust budget pinned to the full 6000
    /// (no pressure-band draws below slot 3000), the camera parked on the
    /// burst position so the off-screen cull never fires, and the
    /// `UpdatePlayers` stream pinned to `seed`.
    fn rig(seed: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("dust".into(), 64, 128);
        app.gfx_quality.max_dust_to_draw = 6000;
        app.camera = Vec2::new(320.0, 640.0);
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        app
    }

    fn burst(kind: ExtraJump) -> JumpDust {
        JumpDust {
            kind,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 3.0,
            vy: -5.01,
            cx: 310.0,
            cy: 621.0,
        }
    }

    fn live(app: &App) -> Vec<&tr_core::particles::Dust> {
        app.dust.slots().iter().filter(|d| d.active).collect()
    }

    fn stream_of(app: &mut App) -> UnifiedRandom {
        app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone()
    }

    /// The stream state after `n` draws from `seed` — the draw-count contract
    /// (the `bow_launch` witness pattern). Every `NewDust` here spends the six
    /// default draws (`Dust.cs:153-193`); no pressure band below slot 3000.
    fn draws_after(seed: i32, n: usize) -> UnifiedRandom {
        let mut rng = UnifiedRandom::new(seed);
        for _ in 0..n {
            rng.next();
        }
        rng
    }

    /// The tick-side snapshot (`player.rs`'s jump chain): a triggered arm
    /// queues one [`JumpDust`] taken AFTER the ascent write (`vy` = the
    /// negative jump speed, `Player.cs:20816` et al.), one-shot on the take;
    /// the ground jump and the unarmed fall-through stay silent.
    #[test]
    fn w4_d51_the_jump_chain_queues_the_post_velocity_snapshot() {
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        // The pin keeps the option through `clear_stale_jumps`; one idle
        // airborne tick sets the release latch (`Player.cs:21026-21031`).
        p.pinned_jumps[ExtraJump::Cloud.index()] = true;
        p.update(&w, crate::player::PlayerInput::default());
        p.can_jump_again[ExtraJump::Cloud.index()] = true;
        p.update(&w, crate::player::PlayerInput { jump: true, ..Default::default() });
        let b = p.take_jump_dust().expect("the Cloud arm queues its burst");
        assert_eq!(b.kind, ExtraJump::Cloud);
        // The box/centre ride the tick-moment position (the integration
        // after the jump section moves the body before the assert).
        assert_eq!(b.cx - b.x, b.width * 0.5);
        assert_eq!(b.cy - b.y, b.height * 0.5);
        assert!(b.width > 0.0 && b.height > 0.0);
        assert!(b.vy < 0.0, "the snapshot rides the ascent write: {}", b.vy);
        assert!(p.take_jump_dust().is_none(), "one-shot");
        // The unarmed fall-through (`:20746`) clears the flag silently.
        let mut p = Player::spawn(&w);
        p.update(&w, crate::player::PlayerInput::default());
        p.update(&w, crate::player::PlayerInput { jump: true, ..Default::default() });
        assert!(p.take_jump_dust().is_none(), "no option, no burst");
    }

    /// `:20787-20806` — Sandstorm / Blizzard play no dust: zero draws, zero
    /// slots.
    #[test]
    fn w4_d51_the_storm_jumps_spend_no_draws() {
        for kind in [ExtraJump::Sandstorm, ExtraJump::Blizzard] {
            let mut app = rig(0x5EED);
            drain_jump_dust(&mut app, burst(kind));
            assert!(live(&app).is_empty());
            assert_eq!(stream_of(&mut app), draws_after(0x5EED, 0));
        }
    }

    /// `:20818-20836` — the Sail: 30 type-253 puffs, 12 draws each (6 default
    /// + 3 post + 3 dead `vector` draws), the even/odd X arms and the
    /// `scale*0.7` velocity fold.
    #[test]
    fn w4_d51_the_sail_replays_thirty_puffs_in_order() {
        let mut app = rig(0x5EED);
        let b = burst(ExtraJump::Sail);
        drain_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 30);
        // The mirror: an independent stream walks the same draw sequence.
        let mut rng = UnifiedRandom::new(0x5EED);
        for (l, d) in slots.iter().enumerate() {
            assert_eq!(d.dust_type, 253);
            assert_eq!(d.alpha, 100);
            assert!(d.no_gravity);
            let px = b.x + rng.next_max(20 - 4) as f32 + 4.0;
            let py = b.y + b.height + rng.next_max(12 - 4) as f32 + 4.0;
            assert_eq!(d.position, Vec2::new(px, py));
            let mut vx = rng.next_range(-20, 21) as f32 * 0.1 + b.vx * 0.3;
            let mut vy = rng.next_range(-20, 21) as f32 * 0.1 + b.vy * 0.3;
            let _ = rng.next_max(3);
            let mut scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * 1.5;
            if l % 2 == 0 {
                vx += rng.next_range(30, 71) as f32 * 0.1;
            } else {
                vx -= rng.next_range(30, 71) as f32 * 0.1;
            }
            vy += rng.next_range(-10, 31) as f32 * 0.1;
            scale += rng.next_range(-10, 41) as f32 * 0.01;
            assert_eq!(d.scale, scale);
            assert_eq!(d.velocity, Vec2::new(vx, vy) * (scale * 0.7));
            let _ = rng.next_range(-100, 101);
            let _ = rng.next_range(-100, 101);
            let _ = rng.next_max(81);
        }
        assert_eq!(stream_of(&mut app), rng, "the mirror walks the stream");
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 30 * 12));
    }

    /// `:20849-20854` — the Fart: 10 type-188 puffs at `(x-34, y+height-16)`,
    /// box 102×32, the deterministic velocity post-arms, 6 draws each.
    #[test]
    fn w4_d51_the_fart_puffs_take_the_post_arms() {
        let mut app = rig(0x5EED);
        let b = burst(ExtraJump::Fart);
        drain_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 10);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 188);
            let px = b.x - 34.0 + rng.next_max(102 - 4) as f32 + 4.0;
            let py = b.y + b.height - 16.0 + rng.next_max(32 - 4) as f32 + 4.0;
            assert_eq!(d.position, Vec2::new(px, py));
            let vx = rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5;
            let vy = rng.next_range(-20, 21) as f32 * 0.1 + b.vy * 0.5;
            let _ = rng.next_max(3);
            let _ = rng.next_range(-20, 21);
            assert_eq!(d.velocity.x, vx * 0.5 - b.vx * 0.1);
            assert_eq!(d.velocity.y, vy * 0.5 - b.vy * 0.3);
            assert!(!d.no_gravity, "the Fart arm sets no noGravity");
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 10 * 6));
    }

    /// `:20874-20891` / `:20902-20919` / `:20930-20947` — the mount rings:
    /// one phase draw, then 5×14 dusts; the Unicorn rolls its type per dust
    /// (9 draws), Goat/Basilisk are fixed (8 draws). The rotated ring
    /// position/velocity is deterministic; scale/fadeIn ride the stream.
    #[test]
    fn w4_d51_the_mount_rings_rotate_from_one_phase_draw() {
        for (kind, fixed_ty, per_dust) in [
            (ExtraJump::Unicorn, None, 9usize),
            (ExtraJump::WallOfFleshGoat, Some(6), 8),
            (ExtraJump::Basilisk, Some(31), 8),
        ] {
            let mut app = rig(0x5EED);
            let b = burst(kind);
            drain_jump_dust(&mut app, b);
            let slots = live(&app);
            assert_eq!(slots.len(), 70);
            let mut rng = UnifiedRandom::new(0x5EED);
            let phase = std::f32::consts::PI * 2.0 * rng.next_float();
            let center = Vec2::new(b.cx, b.cy);
            let mut k = 0;
            for n in 0..5 {
                for num13 in 0..14 {
                    let d = slots[k];
                    k += 1;
                    let ty = match fixed_ty {
                        Some(t) => t,
                        None => [176, 177, 179][rng.next_max(3) as usize],
                    };
                    assert_eq!(d.dust_type, ty);
                    // The six default draws (0×0 box folds to 5×5).
                    let _ = rng.next_max(1);
                    let _ = rng.next_max(1);
                    let _ = rng.next_range(-20, 21);
                    let _ = rng.next_range(-20, 21);
                    let _ = rng.next_max(3);
                    let _ = rng.next_range(-20, 21);
                    let angle =
                        (num13 as f32) * (std::f32::consts::PI * 2.0) / 14.0 + phase;
                    let mut v3 =
                        crate::bow_launch::rotated_by(Vec2::new(0.0, 1.0), angle as f64);
                    v3 *= 0.2 * n as f32;
                    assert_eq!(d.position, center + v3 * Vec2::new(50.0, 20.0));
                    assert_eq!(d.velocity, v3 + Vec2::new(0.0, 4.0));
                    assert!(d.no_gravity);
                    assert_eq!(d.scale, 1.0 + rng.next_float() * 0.8);
                    assert_eq!(d.fade_in, rng.next_float() * 2.0);
                }
            }
            assert_eq!(stream_of(&mut app), rng);
            assert_eq!(stream_of(&mut app), draws_after(0x5EED, 1 + 70 * per_dust));
        }
    }

    /// `:20960-20972` — the Santank: 15 pairs (type 4 then type 6), the
    /// type-6 `velocity.Y--`, and the even-iteration fadeIn draw only. The
    /// type-6 tweak chain (`Dust.cs:214-220`) adds one `Next(-10,6)` draw.
    #[test]
    fn w4_d51_the_santank_pairs_alternate_the_fade_draw() {
        let mut app = rig(0x5EED);
        let b = burst(ExtraJump::Santank);
        drain_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 30);
        let mut rng = UnifiedRandom::new(0x5EED);
        let pos = Vec2::new(b.x - 34.0, b.y + b.height - 16.0);
        for num21 in 0..15 {
            // The type-4 puff: 6 default draws, no tweak.
            let d = slots[num21 * 2];
            assert_eq!(d.dust_type, 4);
            assert_eq!(d.color, Color::new(250, 230, 230, 150));
            let px = pos.x + rng.next_max(98) as f32 + 4.0;
            let py = pos.y + rng.next_max(28) as f32 + 4.0;
            assert_eq!(d.position, Vec2::new(px, py));
            let vx = rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5;
            let vy = rng.next_range(-20, 21) as f32 * 0.1 + b.vy * 0.5;
            let _ = rng.next_max(3);
            let _ = rng.next_range(-20, 21);
            assert_eq!(d.velocity.x, vx * 0.5 - b.vx * 0.1);
            assert_eq!(d.velocity.y, vy * 0.5 - b.vy * 0.3);
            assert!(d.no_gravity);
            // The type-6 puff: 6 default + 1 tweak draw (`:215`).
            let d = slots[num21 * 2 + 1];
            assert_eq!(d.dust_type, 6);
            assert_eq!(d.alpha, 20);
            let _ = rng.next_max(98);
            let _ = rng.next_max(28);
            let mut vx = rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5;
            let _ = rng.next_range(-20, 21);
            let _ = rng.next_max(3);
            let _ = rng.next_range(-20, 21);
            let tweak_y = rng.next_range(-10, 6) as f32 * 0.1;
            vx *= 0.3;
            assert_eq!(d.velocity.x, vx);
            assert_eq!(d.velocity.y, tweak_y - 1.0, "the Y-- post-arm");
            if num21 % 2 == 0 {
                assert_eq!(d.fade_in, rng.next_float() * 2.0);
            } else {
                assert_eq!(d.fade_in, 0.0);
            }
        }
        assert_eq!(stream_of(&mut app), rng);
        // 15 × (6 + 7) + 8 even-iteration fadeIn draws.
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 15 * 13 + 8));
    }

    /// `:21007-21012` — the Cloud: 10 type-16 puffs, the Fart shape, 6 draws
    /// each, no noGravity.
    #[test]
    fn w4_d51_the_cloud_puffs_mirror_the_fart_shape() {
        let mut app = rig(0x5EED);
        let b = burst(ExtraJump::Cloud);
        drain_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 10);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 16);
            assert_eq!(d.alpha, 100);
            let _ = rng.next_max(98);
            let _ = rng.next_max(28);
            let vx = rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5;
            let vy = rng.next_range(-20, 21) as f32 * 0.1 + b.vy * 0.5;
            let _ = rng.next_max(3);
            let _ = rng.next_range(-20, 21);
            assert_eq!(d.velocity.x, vx * 0.5 - b.vx * 0.1);
            assert_eq!(d.velocity.y, vy * 0.5 - b.vy * 0.3);
            assert!(!d.no_gravity);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 10 * 6));
    }
}
