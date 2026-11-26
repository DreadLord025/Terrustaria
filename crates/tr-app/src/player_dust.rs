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
//! `Player.cs:20818-21012`), the `DashMovement` lane (`:21259-21512`) and the
//! fast-run trail (`SpawnFastRunParticles`, `:20217-20298`). The regen lanes
//! land in later batches of the same task. The
//! `Gore.NewGore` siblings of the Fart / Santank / Cloud arms
//! (`:20855-20863`, `:20989-20993`, `:21013-21020`) and the Tabi dash-start
//! gore pair (`:21427-21434`) are the gore lane's (W4-d55) — their `Main.rand`
//! draws are a documented skip here, exactly as the rest of the port treats
//! unported gore (docs/ANALYSIS.md).
//!
//! `gravDir` is the vanilla `1f` (the port has no gravity flip, the
//! `sticky_movement` convention, `Player.cs:1956`), so the `gravDir == -1`
//! height/offset folds collapse to their `+1` arms.
//!
//! W4-f3 scope (third batch, `Player.cs:21654-27933`): the `DoubleJumpVisuals`
//! per-tick trail (`:21892-22021`), the petrify-transition burst (`:25226-25233`)
//! and the `CheckDrowning` bubble (`:23283-23316`). The `Gore.NewGore` sibling of
//! the Sandstorm visual arm (`:21922`) is the gore lane's (W4-d55) — its
//! `Main.rand` draw is a documented skip here.

use crate::app::App;
use crate::player::{DashDust, DoubleJumpDust, ExtraJump, FastRunArm, FastRunDust, FloorDust, JumpDust, SplashArm, SplashDust, StonedDust, VortexStealthDust, WallDust, WingGlideDust};
use glam::Vec2;
use tr_core::particles::Color;

/// One `Dust.NewDust` on the `UpdatePlayers` stream, returning the slot index
/// (the sentinel included — the C# writes `Main.dust[6000]` on a refusal).
pub(crate) fn nd(
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

/// Drain the `DashMovement` dust burst (`Player.cs:21259-21512`). The active
/// trail (`dashDelay < 0`) runs the per-tick arms; the start branch runs the
/// launch burst. The dash-2 loops are `for (…; m < 0; …)` in the source —
/// dead, zero draws. The dash-4 (Ascents) arm has no granter in the port yet
/// (`dashType 4` is never armed), so it stays unqueued; its drain arm lands
/// with the granter.
pub(crate) fn drain_dash_dust(app: &mut App, b: DashDust) {
    if b.active {
        match b.dash {
            1 => dash_trail_tabi(app, &b),
            3 => dash_trail_solar(app, &b),
            5 => dash_trail_vortex(app, &b),
            6 => dash_trail_chillet(app, &b),
            _ => {}
        }
    } else {
        match b.dash {
            1 => dash_start_tabi(app, &b),
            3 => dash_start_solar(app, &b),
            5 => dash_start_vortex(app, &b),
            _ => {}
        }
    }
}

/// `DelegateMethods.Mount.BatDashDust` / `PixieDashDust`
/// (`DelegateMethods.cs:150-188`, identical bodies; `Mount.cs:2422`/`:2485`
/// arm them on mounts 56/61): even iterations deactivate the slot, odd ones
/// re-roll the position inside the player hitbox
/// (`Utils.NextVector2FromRectangle`, `Utils.cs:1256-1259` — two NextFloat
/// draws) and shrink the scale ×0.75.
fn mount_dash_adjust(app: &mut App, b: &DashDust, idx: usize, i: i32) {
    if b.mount != 56 && b.mount != 61 {
        return;
    }
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    if i % 2 == 0 {
        app.dust.slot_mut(idx).active = false;
        return;
    }
    let px = b.x + rng.next_float() * b.width;
    let py = b.y + rng.next_float() * b.height;
    let d = app.dust.slot_mut(idx);
    d.position = Vec2::new(px, py);
    d.scale *= 0.75;
}

/// `:21259-21270` — the Tabi trail: 2 type-31 puffs. `num18 = height/42`
/// scales the box; airborne (`vy != 0`) spawns at the mid-body with the full
/// box height, grounded at the feet with half. Post: `velocity *= 0.1`,
/// `scale *= 1 + Next(20)*0.01`, then the mount delegate.
fn dash_trail_tabi(app: &mut App, b: &DashDust) {
    let num18 = b.height / 42.0;
    let num19 = 4.0 * num18;
    let num20 = (16.0 * num18) as i32;
    for l in 0..2 {
        let (pos, h) = if b.vy != 0.0 {
            (Vec2::new(b.x, b.y + b.height / 2.0 - num19 * 2.0), num20)
        } else {
            (Vec2::new(b.x, b.y + b.height - num19), num20 / 2)
        };
        let idx = nd(app, pos, b.width as i32, h, 31, 0.0, 0.0, 100, Color::default(), 1.4);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.velocity *= 0.1;
        d.scale *= 1.0 + rng.next_max(20) as f32 * 0.01;
        mount_dash_adjust(app, b, idx, l);
    }
}

/// `:21286-21302` — the Solar trail: 4 type-6 puffs at `(x, y + 4*num18)`,
/// box `width × (height - 8*num18)`. Post: `velocity *= 0.1`, the scale roll,
/// `noGravity`, and a `Next(2) == 0` fadeIn 0.5. The type-6 tweak chain
/// (`Dust.cs:214-220`) rides inside `new_dust`.
fn dash_trail_solar(app: &mut App, b: &DashDust) {
    let num18 = b.height / 42.0;
    let num25 = 4.0 * num18;
    let num26 = (8.0 * num18) as i32;
    for n in 0..4 {
        let idx = nd(
            app,
            Vec2::new(b.x, b.y + num25),
            b.width as i32,
            b.height as i32 - num26,
            6,
            0.0,
            0.0,
            100,
            Color::default(),
            1.7,
        );
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.velocity *= 0.1;
        d.scale *= 1.0 + rng.next_max(20) as f32 * 0.01;
        d.no_gravity = true;
        if rng.next_max(2) == 0 {
            d.fade_in = 0.5;
        }
        mount_dash_adjust(app, b, idx, n);
    }
}

/// `:21328-21343` — the Vortex trail: 2 puffs, type rolled per puff from
/// `{68, 69, 70}` (`NextFromList`, `Utils.cs:1213`), the Tabi box shape,
/// scale 1. Post: `velocity *= 0.2`, the scale roll, `fadeIn = 0.5 +
/// Next(20)*0.01`, `noGravity`.
fn dash_trail_vortex(app: &mut App, b: &DashDust) {
    let num18 = b.height / 42.0;
    let num32 = 4.0 * num18;
    let num33 = (16.0 * num18) as i32;
    for num34 in 0..2 {
        let ty = {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            [68, 69, 70][rng.next_max(3) as usize]
        };
        let (pos, h) = if b.vy != 0.0 {
            (Vec2::new(b.x, b.y + b.height / 2.0 - num32 * 2.0), num33)
        } else {
            (Vec2::new(b.x, b.y + b.height - num32), num33 / 2)
        };
        let idx = nd(app, pos, b.width as i32, h, ty, 0.0, 0.0, 100, Color::default(), 1.0);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.velocity *= 0.2;
        d.scale *= 1.0 + rng.next_max(20) as f32 * 0.01;
        d.fade_in = 0.5 + rng.next_max(20) as f32 * 0.01;
        d.no_gravity = true;
        mount_dash_adjust(app, b, idx, num34);
    }
}

/// `Color.Lerp(value1, value2, amount)` — XNA's per-channel byte lerp,
/// truncated (the `render::pylons::lerp_bytes` shape, all four channels).
pub(crate) fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// `:21344-21372` — the Chillet trail: the direction-flip abort is the tick's
/// (it queues nothing); the surviving arm feeds the centre light
/// (`Lighting.AddLight`, no draws — the `entity_lights` convention) and spawns
/// 8 type-306 puffs at `(x + direction*80, y + 12)`, box `width × (height-20)`.
/// Post: `velocity *= 1.5`, `scale *= 1 + Next(40)*0.01`, `fadeIn = 0.5 +
/// Next(20)*0.01`, `noGravity`, `velocity.X += direction*1.5`, then the
/// `NextFloat` colour cube-lerp toward White (the warm base on mounts 63/65).
fn dash_trail_chillet(app: &mut App, b: &DashDust) {
    // `:21354` — the mount-type colour pick; `Lighting.AddLight(Vector2, …)`
    // truncates the centre per axis (`Lighting.cs:140-143`).
    let warm = b.mount == 63 || b.mount == 65;
    let rgb = if warm { [0.6, 0.3, 0.1] } else { [0.5, 0.1, 0.6] };
    let cx = b.x + b.width * 0.5;
    let cy = b.y + b.height * 0.5;
    app.entity_lights.add((cx / 16.0) as i32, (cy / 16.0) as i32, rgb);
    let base = if warm {
        // `new Color(1f, 0.7f, 0.5f)` — the Vector3 ctor's `(byte)(v*255)`.
        Color::new(255, (0.7f32 * 255.0) as u8, (0.5f32 * 255.0) as u8, 255)
    } else {
        Color::new((0.9f32 * 255.0) as u8, (0.7f32 * 255.0) as u8, 255, 255)
    };
    for num36 in 0..8 {
        let idx = nd(
            app,
            Vec2::new(b.x + b.facing as f32 * 80.0, b.y + 12.0),
            b.width as i32,
            b.height as i32 - 20,
            306,
            0.0,
            0.0,
            100,
            Color::default(),
            1.0,
        );
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.velocity *= 1.5;
        d.scale *= 1.0 + rng.next_max(40) as f32 * 0.01;
        d.fade_in = 0.5 + rng.next_max(20) as f32 * 0.01;
        d.no_gravity = true;
        d.velocity.x += b.facing as f32 * 1.5;
        let num38 = rng.next_float();
        d.color = lerp_color(base, Color::WHITE, num38 * num38 * num38);
        mount_dash_adjust(app, b, idx, num36);
    }
}

/// The dash-start burst's shared post-arms (`:21422-21425`, `:21477-21480`):
/// the ±5 position jitter (two draws), `velocity *= 0.2`, the scale roll.
fn start_jitter(app: &mut App, idx: usize) {
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let d = app.dust.slot_mut(idx);
    d.position.x += rng.next_range(-5, 6) as f32;
    d.position.y += rng.next_range(-5, 6) as f32;
    d.velocity *= 0.2;
    d.scale *= 1.0 + rng.next_max(20) as f32 * 0.01;
}

/// `:21419-21426` — the Tabi start: 20 type-31 puffs over the whole body box,
/// scale 2. The two `Gore.NewGore` siblings (`:21427-21434`) are the gore
/// lane's (W4-d55, skip — their draws included).
fn dash_start_tabi(app: &mut App, b: &DashDust) {
    for _ in 0..20 {
        let idx = nd(
            app,
            Vec2::new(b.x, b.y),
            b.width as i32,
            b.height as i32,
            31,
            0.0,
            0.0,
            100,
            Color::default(),
            2.0,
        );
        start_jitter(app, idx);
    }
}

/// `:21474-21484` — the Solar start: 20 type-6 puffs, the Tabi shape plus
/// `noGravity` and a flat `fadeIn = 0.5` (no draws beyond the jitter).
fn dash_start_solar(app: &mut App, b: &DashDust) {
    for _ in 0..20 {
        let idx = nd(
            app,
            Vec2::new(b.x, b.y),
            b.width as i32,
            b.height as i32,
            6,
            0.0,
            0.0,
            100,
            Color::default(),
            2.0,
        );
        start_jitter(app, idx);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.fade_in = 0.5;
    }
}

/// `:21500-21510` — the Vortex start: 20 puffs, type rolled per puff from
/// `{68, 69, 70}`, scale 1.5. Post: the ±5 jitter, then the velocity REPLACED
/// by `base.DirectionTo(dust.position) * 2` (`Entity.cs:205-208` — the
/// centre-relative normalise), the scale roll, `fadeIn = 0.5 + Next(20)*0.01`,
/// `noGravity`.
fn dash_start_vortex(app: &mut App, b: &DashDust) {
    let center = Vec2::new(b.x + b.width * 0.5, b.y + b.height * 0.5);
    for _ in 0..20 {
        let ty = {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            [68, 69, 70][rng.next_max(3) as usize]
        };
        let idx = nd(
            app,
            Vec2::new(b.x, b.y),
            b.width as i32,
            b.height as i32,
            ty,
            0.0,
            0.0,
            100,
            Color::default(),
            1.5,
        );
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.position.x += rng.next_range(-5, 6) as f32;
        d.position.y += rng.next_range(-5, 6) as f32;
        // `Vector2.Normalize` — the zero vector stays zero (no draws).
        let delta = d.position - center;
        d.velocity = if delta == Vec2::ZERO { Vec2::ZERO } else { delta.normalize() * 2.0 };
        d.scale *= 1.0 + rng.next_max(20) as f32 * 0.01;
        d.fade_in = 0.5 + rng.next_max(20) as f32 * 0.01;
        d.no_gravity = true;
    }
}

/// Drain the fast-run trail (`Player.SpawnFastRunParticles`,
/// `Player.cs:20217-20298`). The tick picked the arm and snapshotted the
/// box/velocity; the foot-line offset is `y + height` (the gravDir == -1
/// fold collapses to 0). Called right after the footstep sound drain so the
/// sound-then-dust order matches the source.
pub(crate) fn drain_fast_run(app: &mut App, b: FastRunDust) {
    match b.arm {
        FastRunArm::WingsRocket => fast_run_wings_rocket(app, &b),
        FastRunArm::Sail => fast_run_sail(app, &b),
        FastRunArm::Desert => fast_run_desert(app, &b),
        FastRunArm::Cold => fast_run_cold(app, &b),
        FastRunArm::Fairy => fast_run_fairy(app, &b),
        FastRunArm::Hellfire => fast_run_hellfire(app, &b),
        FastRunArm::Plain => fast_run_plain(app, &b),
    }
}

/// The foot-line spawn box shared by most arms: `(x-4, y+height)`,
/// `width+8 × 4`, speed `(-vx*0.5, vy*0.5)`.
fn foot_pos(b: &FastRunDust) -> Vec2 {
    Vec2::new(b.x - 4.0, b.y + b.height)
}

/// `:20217-20225` — wings-3 / rocket-6: two type-186 puffs, alpha 50, scale
/// 1.5; the first's velocity ×0.025, the second's ×0.2.
fn fast_run_wings_rocket(app: &mut App, b: &FastRunDust) {
    for fold in [0.025f32, 0.2] {
        let idx = nd(
            app,
            foot_pos(b),
            b.width as i32 + 8,
            4,
            186,
            -b.vx * 0.5,
            b.vy * 0.5,
            50,
            Color::default(),
            1.5,
        );
        app.dust.slot_mut(idx).velocity *= fold;
    }
}

/// `:20226-20240` — sailDash: four type-253 puffs over the whole body box at
/// `(x-4, y)`, alpha 100, scale 1.5. Post: `noGravity`, velocity ×0.2 per
/// axis, `scale += Next(-5,3)*0.1`, then the DEAD `vector` draws
/// (`Next(-100,101)`×2 + `Next(81)` — normalised/scaled, never read).
fn fast_run_sail(app: &mut App, b: &FastRunDust) {
    for _ in 0..4 {
        let idx = nd(
            app,
            Vec2::new(b.x - 4.0, b.y),
            b.width as i32 + 8,
            b.height as i32,
            253,
            -b.vx * 0.5,
            b.vy * 0.5,
            100,
            Color::default(),
            1.5,
        );
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity.x *= 0.2;
        d.velocity.y *= 0.2;
        d.scale += rng.next_range(-5, 3) as f32 * 0.1;
        let _ = rng.next_range(-100, 101);
        let _ = rng.next_range(-100, 101);
        let _ = rng.next_max(81);
    }
}

/// `:20241-20247` — desertDash: one type-32 `NewDustDirect` at the foot line,
/// default alpha/scale. Post: velocity ×0.2, `velocity.Y -= gravDir*2`
/// (gravDir 1 → −2). The Direct wrapper zeroes a NaN velocity (never here).
fn fast_run_desert(app: &mut App, b: &FastRunDust) {
    let ctx = app.spawn_ctx();
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let idx = app.dust.new_dust_direct(
        rng,
        &ctx,
        &app.world,
        foot_pos(b),
        b.width as i32 + 8,
        4,
        32,
        -b.vx * 0.5,
        b.vy * 0.5,
        0,
        Color::default(),
        1.0,
    );
    let d = app.dust.slot_mut(idx);
    d.velocity *= 0.2;
    d.velocity.y -= 2.0;
}

/// `:20248-20260` — coldDash: two type-76 puffs, alpha 0, scale 1.35. The
/// first spawns at `(x, y+height+gfxOffY)` box `width/2 × 6`, the second at
/// `(x+width/2, …)`. Post: `scale *= 1 + Next(20,40)*0.01`, `noGravity`,
/// `noLight`, velocity ×0.001 then `velocity.Y -= 0.003`.
fn fast_run_cold(app: &mut App, b: &FastRunDust) {
    for j in 0..2 {
        let px = if j != 0 { b.x + b.width / 2.0 } else { b.x };
        let idx = nd(
            app,
            Vec2::new(px, b.y + b.height + b.gfx_off_y),
            (b.width / 2.0) as i32,
            6,
            76,
            0.0,
            0.0,
            0,
            Color::default(),
            1.35,
        );
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.scale *= 1.0 + rng.next_range(20, 40) as f32 * 0.01;
        d.no_gravity = true;
        d.no_light = true;
        d.velocity *= 0.001;
        d.velocity.y -= 0.003;
    }
}

/// `:20261-20282` — fairyBoots: ONE type roll from `{61,61,61,242,64,63}`
/// (`NextFromList`, `Next(6)`) before the loop, then two puffs (k=1 scale 1.5,
/// k=2 scale 1.0), alpha 0. Post: velocity ×1.5; on k=2 `position += velocity`;
/// `noGravity`, `noLightEmittance`.
fn fast_run_fairy(app: &mut App, b: &FastRunDust) {
    let ty = {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        [61, 61, 61, 242, 64, 63][rng.next_max(6) as usize]
    };
    for k in 1..3 {
        let scale = if k == 2 { 1.0 } else { 1.5 };
        let idx = nd(
            app,
            foot_pos(b),
            b.width as i32 + 8,
            4,
            ty,
            -b.vx * 0.5,
            b.vy * 0.5,
            0,
            Color::default(),
            scale,
        );
        let d = app.dust.slot_mut(idx);
        d.velocity *= 1.5;
        if k == 2 {
            d.position += d.velocity;
        }
        d.no_gravity = true;
        d.no_light_emittance = true;
    }
}

/// `:20283-20291` — hellfireTreads: one type-6 puff, alpha 50, scale 2. Post:
/// `velocity.X *= 0.2`, `velocity.Y = -1.5 - NextFloat*0.5`, `fadeIn = 0.5`,
/// `noGravity`. The type-6 tweak chain rides inside `new_dust`.
fn fast_run_hellfire(app: &mut App, b: &FastRunDust) {
    let idx = nd(
        app,
        foot_pos(b),
        b.width as i32 + 8,
        4,
        6,
        -b.vx * 0.5,
        b.vy * 0.5,
        50,
        Color::default(),
        2.0,
    );
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let d = app.dust.slot_mut(idx);
    d.velocity.x *= 0.2;
    d.velocity.y = -1.5 - rng.next_float() * 0.5;
    d.fade_in = 0.5;
    d.no_gravity = true;
}

/// `:20292-20298` — the plain foot puff: one type-16, alpha 50, scale 1.5.
/// Post: velocity ×0.2 per axis.
fn fast_run_plain(app: &mut App, b: &FastRunDust) {
    let idx = nd(
        app,
        foot_pos(b),
        b.width as i32 + 8,
        4,
        16,
        -b.vx * 0.5,
        b.vy * 0.5,
        50,
        Color::default(),
        1.5,
    );
    let d = app.dust.slot_mut(idx);
    d.velocity.x *= 0.2;
    d.velocity.y *= 0.2;
}

// ---- W4-f3: the DoubleJumpVisuals per-tick trail (`Player.cs:21892-22021`) --

/// Drain the `DoubleJumpVisuals` dust trail (`Player.cs:21892-22021`). Called
/// after the dash dust on the `UpdatePlayers` stream (the C# runs
/// `DoubleJumpVisuals` at `:26767`, right after `DashMovement` `:26757`). The
/// tick snapshotted the box/velocity/jump-counter at the C# read moment; each
/// arm replays its `Dust.NewDust` chain in source order. The performing flag is
/// the single active arm (the C#'s `isPerformingJump_*` flags are mutually
/// exclusive — one extra jump per ascent).
pub(crate) fn drain_double_jump_dust(app: &mut App, b: DoubleJumpDust) {
    match b.kind {
        ExtraJump::Cloud => dj_cloud(app, &b),
        ExtraJump::Sandstorm => dj_sandstorm(app, &b),
        ExtraJump::Fart => dj_fart(app, &b),
        ExtraJump::Unicorn => dj_unicorn(app, &b),
        ExtraJump::Sail => dj_sail(app, &b),
        ExtraJump::Blizzard => dj_blizzard(app, &b),
        // Santank / WallOfFleshGoat / Basilisk have no DoubleJumpVisuals arm.
        _ => {}
    }
}

/// `:21894-21904` — the Cloud's one type-16 puff, box `(width+8)×4` at
/// `(x-4, y+height)`, speed `(-vx*0.5, vy*0.5)`, alpha 100, scale 1.5. Post:
/// the Fart-style velocity arms (no RNG).
fn dj_cloud(app: &mut App, b: &DoubleJumpDust) {
    let pos = Vec2::new(b.x - 4.0, b.y + b.height);
    let idx = nd(
        app,
        pos,
        b.width as i32 + 8,
        4,
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

/// `:21905-21926` — the Sandstorm's 3 type-124 puffs, box `width×32` at
/// `(x, y+height/2)`, speed `(vx*0.3, vy*0.3)`, alpha 150, scale `1*num4`
/// where `num4 = (jump/75 + 1)/2`. Post: `velocity *= 0.5*num4`,
/// `fadeIn = 1.5*num4`. The `Gore.NewGore` sibling (`:21922`, the
/// `miscCounter % 3 == 0` arm) is the gore lane's (W4-d55 body); its type
/// ARGUMENT draw `Next(220,223)` rides this stream (KEEP-E1, the seal).
fn dj_sandstorm(app: &mut App, b: &DoubleJumpDust) {
    let num4 = (b.jump as f32 / 75.0 + 1.0) / 2.0;
    // `:21915`: `(float)(num3 / 2)` — INTEGER half (the wall-corner class;
    // gravDir 1 so num3 = height). KEEP-E1's int form (W4-f3 seal).
    let pos = Vec2::new(b.x, b.y + (b.height as i32 / 2) as f32);
    for _ in 0..3 {
        let idx = nd(
            app,
            pos,
            b.width as i32,
            32,
            124,
            b.vx * 0.3,
            b.vy * 0.3,
            150,
            Color::default(),
            num4,
        );
        let d = app.dust.slot_mut(idx);
        d.velocity *= 0.5 * num4;
        d.fade_in = 1.5 * num4;
    }
    // W4-f3 seal (KEEP-E1) — `:21920-21925`: the Sandstorm gore sibling. When
    // the `miscCounter % 3 == 0` gate fires, the `Gore.NewGore` type ARGUMENT
    // `Main.rand.Next(220,223)` (`:21922`) draws on `UpdatePlayers` BEFORE the
    // gore body. The body + its `velocity`/`alpha` post-writes are W4-d55's
    // lane (the gore body is unported), but the argument draw is NOT d55's —
    // it rides this stream and must not be skipped (determinism invariant).
    if b.sandstorm_gore_gate {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let _ = rng.next_range(220, 223);
    }
}

/// `:21927-21938` — the Fart's one type-188 puff, the Cloud shape plus a final
/// `velocity *= 0.5` (no RNG).
fn dj_fart(app: &mut App, b: &DoubleJumpDust) {
    let pos = Vec2::new(b.x - 4.0, b.y + b.height);
    let idx = nd(
        app,
        pos,
        b.width as i32 + 8,
        4,
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
    d.velocity *= 0.5;
}

/// `:21939-21947` — the Unicorn's one puff of a rolled type
/// (`SelectRandom(176,177,179)` — a `Next(3)` BEFORE the `NewDust` internals),
/// box `width×height` at `position`. Post: `velocity = Zero`, `noGravity`,
/// `scale = 0.5 + NextFloat*0.8`, `fadeIn = 1 + NextFloat*2`.
fn dj_unicorn(app: &mut App, b: &DoubleJumpDust) {
    let ty = {
        let r = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        [176, 177, 179][r.next_max(3) as usize]
    };
    let idx = nd(
        app,
        Vec2::new(b.x, b.y),
        b.width as i32,
        b.height as i32,
        ty,
        0.0,
        0.0,
        0,
        Color::default(),
        1.0,
    );
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let d = app.dust.slot_mut(idx);
    d.velocity = Vec2::ZERO;
    d.no_gravity = true;
    d.scale = 0.5 + rng.next_float() * 0.8;
    d.fade_in = 1.0 + rng.next_float() * 2.0;
}

/// `:21948-21977` — the Sail's 1-or-2 type-253 puffs (`num9 = 2` while
/// `jump > 0`, else 1), box `width×12` at `(x, y+height-6)`, speed
/// `(vx*0.3, vy*0.3)`, alpha 100, scale 1.5. Per puff: `scale += Next(-5,3)*0.1`,
/// then `scale *= 0.8` (jump <= 0) or `velocity -= base.velocity/5` (jump > 0),
/// `noGravity`, then the DEAD `vector` draws `Next(-100,101)`×2 + `Next(81)`.
fn dj_sail(app: &mut App, b: &DoubleJumpDust) {
    let num9 = if b.jump > 0 { 2 } else { 1 };
    let pos = Vec2::new(b.x, b.y + b.height - 6.0);
    for _ in 0..num9 {
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
        d.scale += rng.next_range(-5, 3) as f32 * 0.1;
        if b.jump <= 0 {
            d.scale *= 0.8;
        } else {
            d.velocity -= Vec2::new(b.vx, b.vy) / 5.0;
        }
        d.no_gravity = true;
        // The dead `vector` local (`:21973-21975`): three draws, no effect.
        let _ = rng.next_range(-100, 101);
        let _ = rng.next_range(-100, 101);
        let _ = rng.next_max(81);
    }
}

/// `:21978-22020` — the Blizzard's three type-76 loops (2 + 3 + 3), box
/// `width×12` at `(x, y+height-6)`, speed `(vx*0.3, vy*0.3)`. Loop 1:
/// `velocity *= 0.1`, `± base.velocity*0.03` (k==0 add / else subtract),
/// `- base.velocity*0.1`, `noGravity`, `noLight`. Loops 2/3: `fadeIn = 1.5`,
/// `velocity *= 0.6`, `± base.velocity*0.8`, `noGravity`, `noLight`.
fn dj_blizzard(app: &mut App, b: &DoubleJumpDust) {
    let pos = Vec2::new(b.x, b.y + b.height - 6.0);
    let v = Vec2::new(b.vx, b.vy);
    for k in 0..2 {
        let idx = nd(app, pos, b.width as i32, 12, 76, b.vx * 0.3, b.vy * 0.3, 0, Color::default(), 1.0);
        let d = app.dust.slot_mut(idx);
        d.velocity *= 0.1;
        if k == 0 {
            d.velocity += v * 0.03;
        } else {
            d.velocity -= v * 0.03;
        }
        d.velocity -= v * 0.1;
        d.no_gravity = true;
        d.no_light = true;
    }
    for _ in 0..3 {
        let idx = nd(app, pos, b.width as i32, 12, 76, b.vx * 0.3, b.vy * 0.3, 0, Color::default(), 1.0);
        let d = app.dust.slot_mut(idx);
        d.fade_in = 1.5;
        d.velocity *= 0.6;
        d.velocity += v * 0.8;
        d.no_gravity = true;
        d.no_light = true;
    }
    for _ in 0..3 {
        let idx = nd(app, pos, b.width as i32, 12, 76, b.vx * 0.3, b.vy * 0.3, 0, Color::default(), 1.0);
        let d = app.dust.slot_mut(idx);
        d.fade_in = 1.5;
        d.velocity *= 0.6;
        d.velocity -= v * 0.8;
        d.no_gravity = true;
        d.no_light = true;
    }
}

// ---- W4-f3: the petrify-transition burst (`Player.cs:25226-25233`) ----------

// ---- W4-f3: the wall-climb/wallslide dust (`Player.cs:21654-21826`) ---------

/// Drain the wall-climb/wallslide puff (`Player.WallClimbMovement`
/// `Player.cs:21654`/`:21678`/`:21729`, `WallslideMovement` `:21790`/`:21826` —
/// all five sites share the same chain). One type-31 `Dust.NewDust`, box 8×8
/// at the wall-side body corner `(x + width/2 + (width/2 - 4)*slideDir,
/// y + height/2 + (height/2 - 4)*gravDir)` — INTEGER halves (`:21654` casts
/// the int division; the g66 keep-one form), gravDir 1. Post-arms in source
/// order: `slideDir < 0 → position.X -= 10`, the `gravDir < 0 → Y -= 12` fold
/// (dead, gravDir 1), `velocity *= 0.1`, `scale *= 1.2`, `noGravity`, the
/// dropped `cShoe` shader, then the Rat's `scale *= 0.5` (`flag`). The
/// suppression probe (the Rat climbing into a ceiling) ran on the tick side.
pub(crate) fn drain_wall_dust(app: &mut App, b: WallDust) {
    // W4-f3 retire keep-one (g66 248f0ff4d verdict): the C# corner uses
    // INTEGER halves (`(float)(base.width / 2)`, `:21654`) — float halves
    // drift 0.5 px on odd hitboxes. g66's form adopted.
    let hw = b.width as i32 / 2;
    let hh = b.height as i32 / 2;
    let pos = Vec2::new(
        b.x + hw as f32 + (hw - 4) as f32 * b.slide_dir as f32,
        b.y + hh as f32 + (hh - 4) as f32,
    );
    let idx = nd(app, pos, 8, 8, 31, 0.0, 0.0, 0, Color::default(), 1.0);
    let d = app.dust.slot_mut(idx);
    if b.slide_dir < 0 {
        d.position.x -= 10.0;
    }
    d.velocity *= 0.1;
    d.scale *= 1.2;
    d.no_gravity = true;
    if b.rat_mount {
        d.scale *= 0.5;
    }
}

// ---- W4-f3: the Vortex-stealth transition ring (`Player.cs:25872-25893`) ----

/// Drain the Vortex-stealth transition ring (`Player.cs:25872-25893`). TWO
/// independent `Next(2)` gates, each drawing only on a 0:
/// * the type-229 outer ring (`:25874-25883`): `UnitY.RotatedByRandom(2π)`
///   (TWO `NextDouble`, left first — `Utils.cs:1697-1700`), `NewDust` at
///   `Center - vector*30` (box 0×0), then `noGravity`, `position = Center -
///   vector*Next(5,11)`, `velocity = vector.RotatedBy(π/2)*4`,
///   `scale = 0.5 + NextFloat`, `fadeIn = 0.5`;
/// * the type-240 inner ring (`:25884-25893`): a fresh spin, `NewDust` at
///   `Center - vector2*30`, `noGravity`, `position = Center - vector2*12`,
///   `velocity = vector2.RotatedBy(-π/2)*2`, `scale = 0.5 + NextFloat`,
///   `fadeIn = 0.5`.
/// The `flag11` gate (the stealth value changed without clamping) ran on the
/// tick side; a refused gate spends NO draws (the C# `if` short-circuits).
pub(crate) fn drain_vortex_stealth(app: &mut App, b: VortexStealthDust) {
    let center = Vec2::new(b.cx, b.cy);
    let tau = 6.2831854820251465f64;
    let half_pi = 1.5707963705062866f64;
    // `:25874`: the outer gate.
    if app.rngs.stream(crate::app::RngKey::UpdatePlayers).next_max(2) == 0 {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let angle = rng.next_double() * tau - rng.next_double() * tau;
        let vector = crate::bow_launch::rotated_by(Vec2::new(0.0, 1.0), angle);
        let idx = nd(app, center - vector * 30.0, 0, 0, 229, 0.0, 0.0, 0, Color::default(), 1.0);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let radius = rng.next_range(5, 11) as f32;
        let scale = 0.5 + rng.next_float();
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.position = center - vector * radius;
        d.velocity = crate::bow_launch::rotated_by(vector, half_pi) * 4.0;
        d.scale = scale;
        d.fade_in = 0.5;
    }
    // `:25884`: the inner gate.
    if app.rngs.stream(crate::app::RngKey::UpdatePlayers).next_max(2) == 0 {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let angle = rng.next_double() * tau - rng.next_double() * tau;
        let vector2 = crate::bow_launch::rotated_by(Vec2::new(0.0, 1.0), angle);
        let idx = nd(app, center - vector2 * 30.0, 0, 0, 240, 0.0, 0.0, 0, Color::default(), 1.0);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let scale = 0.5 + rng.next_float();
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.position = center - vector2 * 12.0;
        d.velocity = crate::bow_launch::rotated_by(vector2, -half_pi) * 2.0;
        d.scale = scale;
        d.fade_in = 0.5;
    }
}

/// Drain the petrify-transition burst (`Player.cs:25226-25233`): 20 type-1
/// puffs over the player box, each followed by a `Next(2)` that sets
/// `noGravity` on a 0. The C# plays `PlaySound(0)` at `position` (`:25225`)
/// BEFORE the loop; the sound is a cited seam (the port's stoned hurt rides
/// `hurt_inner`'s own cue), so the drain replays only the dust chain.
pub(crate) fn drain_stoned_dust(app: &mut App, b: StonedDust) {
    let pos = Vec2::new(b.x, b.y);
    for _ in 0..20 {
        let idx = nd(
            app,
            pos,
            b.width as i32,
            b.height as i32,
            1,
            0.0,
            0.0,
            0,
            Color::default(),
            1.0,
        );
        let roll = app
            .rngs
            .stream(crate::app::RngKey::UpdatePlayers)
            .next_max(2);
        if roll == 0 {
            app.dust.slot_mut(idx).no_gravity = true;
        }
    }
}

// ---- W4-f3: the CheckDrowning bubble (`Player.cs:23283-23316`) --------------

/// Drain the drowning bubble (`Player.cs:23283-23316`): one type-34 puff,
/// box `(width-8)×8` at the mouth position, speed `(0,0)`, alpha 0, scale 1.2.
/// The `Next(20) != 0` gate (`:23283`) is drawn on the tick side (it gates the
/// whole bubble, lava/honey-wet excluded); the type-34 wet-probe kill rides
/// `DustPool::new_dust` verbatim (`Dust.cs:248-250`).
pub(crate) fn drain_drown_bubble(app: &mut App, pos: Vec2, width: f32) {
    nd(app, pos, width as i32 - 8, 8, 34, 0.0, 0.0, 0, Color::default(), 1.2);
}

// ---- W4-f3: MakeFloorDust (`Player.cs:23790-24043`) -------------------------

/// The floor-tile → dust-type map (`Player.cs:23836-23920`), default 76.
fn floor_dust_type(kind: i32, frame_y: i32) -> i32 {
    let platform49 = kind == 19 && frame_y / 18 == 49;
    match kind {
        666 | 712 => 322,
        53 => 32,
        189 => 16,
        19 if platform49 => 16,
        717 => 326,
        718 => 327,
        719 => 328,
        0 => 0,
        123 => 53,
        57 => 36,
        112 => 14,
        234 => 122,
        116 => 51,
        196 | 460 => 108,
        193 | 197 => 4,
        195 | 199 => 5,
        229 => 153,
        371 => 243,
        25 => 37,
        _ => 76,
    }
}

/// The floor-dust allow-list (`Player.cs:23819-23823`): the kinds that spawn
/// dust at all (the platform-49 carve-out included).
fn floor_dust_allowed(kind: i32, frame_y: i32) -> bool {
    let platform49 = kind == 19 && frame_y / 18 == 49;
    matches!(
        kind,
        147 | 25 | 53 | 189 | 0 | 123 | 57 | 112 | 116 | 196 | 193 | 195 | 197
            | 199 | 229 | 234 | 371 | 460 | 666 | 712 | 717 | 718 | 719
    ) || platform49
}

/// Drain the `MakeFloorDust` request (`Player.cs:23790-24043`). The shimmer-block
/// orchestrator arm (`:23792-23818`, tiles 659/667) is a cited seam (the
/// ParticleOrchestrator is unported); the `Item177` landing cue (`:23830`) is an
/// audio seam. The per-iteration gate rolls + the `Dust.NewDust` chain ride the
/// `UpdatePlayers` stream in source order.
pub(crate) fn drain_floor_dust(app: &mut App, b: FloorDust) {
    if !floor_dust_allowed(b.kind, b.frame_y) {
        return;
    }
    let num2 = if b.falling { 20 } else { 1 };
    for _ in 0..num2 {
        // `:23837-23844`: the 666/712 painted-tile break (no draws).
        if (b.kind == 666 || b.kind == 712) && b.paint != 0 {
            break;
        }
        let num3 = floor_dust_type(b.kind, b.frame_y);
        let mut flag3 = true;
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        // `:23921-23944`: the per-type gate rolls (only the matching arm draws).
        if num3 == 32 && rng.next_max(2) == 0 {
            flag3 = false;
        }
        if num3 == 14 && rng.next_max(2) == 0 {
            flag3 = false;
        }
        if num3 == 51 && rng.next_max(2) == 0 {
            flag3 = false;
        }
        if num3 == 36 && rng.next_max(2) == 0 {
            flag3 = false;
        }
        if num3 == 0 && rng.next_max(3) != 0 {
            flag3 = false;
        }
        if num3 == 53 && rng.next_max(3) != 0 {
            flag3 = false;
        }
        // `:23945-23957`: the per-kind colour.
        let color = match b.kind {
            193 => Color::new(30, 100, 255, 100),
            197 => Color::new(97, 200, 255, 100),
            460 => Color::new(100, 150, 130, 100),
            _ => Color::default(),
        };
        // `:23958-23965`: the walking gate (not falling) — `Next(100)`.
        if !b.falling {
            let num4 = b.vx.abs() / 3.0;
            if rng.next_max(100) as f32 > num4 * 100.0 {
                flag3 = false;
            }
        }
        if !flag3 {
            continue;
        }
        // `:23970-23982`: the clamped vx + the walking-effects gate (no draws).
        let num5 = b.vx.clamp(-6.0, 6.0);
        let walking = b.vx.abs() + b.vy.abs() > 1.0;
        if !(walking || b.falling) {
            continue;
        }
        // `:23983`: the NewDust (6 default draws), then the post-arms.
        let pos = Vec2::new(b.x, b.y + b.height - 2.0);
        let idx = nd(app, pos, b.width as i32, 6, num3, 0.0, 0.0, 50, color, 1.0);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        if num3 == 76 {
            d.scale += rng.next_max(3) as f32 * 0.1;
            d.no_light = true;
        }
        if matches!(num3, 16 | 108 | 153 | 326 | 327 | 328) {
            d.scale += rng.next_max(6) as f32 * 0.1;
        }
        if num3 == 37 {
            d.scale += 0.25;
            d.alpha = 50;
        }
        if num3 == 5 {
            d.scale += rng.next_range(2, 8) as f32 * 0.1;
        }
        d.no_gravity = true;
        if num3 == 322 {
            if rng.next_max(4) == 0 {
                d.no_gravity = false;
                d.scale *= 1.1;
            } else {
                d.scale *= 1.2;
            }
        }
        if num2 > 1 {
            d.velocity.x *= 1.2;
            d.velocity.y *= 0.8;
            d.velocity.y -= 1.0;
            d.velocity *= 0.8;
            d.scale += rng.next_max(3) as f32 * 0.1;
            // `:24026`: `(float)(base.width / 2)` — INTEGER half (the same
            // int-division class as the wall-puff corner, g66 248f0ff4d). f3's
            // float half drifted 0.5 px on odd widths; e1's make_floor_dust
            // already uses the int form. KEEP-E1's form (W4-f3 seal).
            d.velocity.x = (d.position.x - (b.x + (b.width as i32 / 2) as f32)) * 0.2;
            if d.velocity.y > 0.0 {
                d.velocity.y *= -1.0;
            }
            d.velocity.x += num5 * 0.3;
        } else {
            d.velocity *= 0.2;
        }
        d.position.x -= num5;
    }
}

// ---- W4-f3: the wing-glide dust trail (`Player.cs:27040-27337`) -------------

/// Drain the wing-glide dust trail (`Player.cs:27040-27337`). The per-wing arms
/// run in source order; the gate rolls ride the `UpdatePlayers` stream even when
/// they refuse. `noLightEmittance` (`wingsLogic != wings`, `:27036`) folds to
/// false (the port's single wing field); the `cWings` shader is a dropped field.
/// The wings-4 arm's `wingFrame` machine + `Item13` cadence (`:27149-27191`) are
/// cited seams (the port's `WingFrame` owns the frame machine).
pub(crate) fn drain_wing_glide(app: &mut App, b: WingGlideDust) {
    // `:27040`: wings 10 — `Next(3) == 0`, one type-76 puff.
    if b.wings == 10 && glide_draw(app, 3) == 0 {
        glide_puff(app, &b, 4, -40, -15.0, 30, 30, 76, 50, 0.6, Color::default(), GlidePost::Fade03);
    }
    // `:27054`: wings 34 — the always-animated gate (vy != 0 on the glide) +
    // `Next(3) == 0`, one type-261 puff.
    if b.wings == 34 && b.vy != 0.0 && glide_draw(app, 3) == 0 {
        glide_puff(app, &b, 4, -40, -15.0, 30, 30, 261, 50, 0.6, Color::default(), GlidePost::Fade03);
    }
    // `:27069`: wings 51 — `Next(5) == 0`, one type-261 puff, colour (230,130,55).
    if b.wings == 51 && glide_draw(app, 5) == 0 {
        glide_puff(app, &b, 4, -40, -15.0, 30, 30, 261, 50, 0.6, Color::new(230, 130, 55, 255), GlidePost::Fade03);
    }
    // `:27092`: wings 9 — `Next(3) == 0`, one type-6 puff (offset 8/-40).
    if b.wings == 9 && glide_draw(app, 3) == 0 {
        glide_puff(app, &b, 8, -40, -15.0, 30, 30, 6, 200, 2.0, Color::default(), GlidePost::NoGrav03);
    }
    // `:27105`: wings 29 — `Next(3) == 0`, one type-6 puff + the `Next(10)` fade.
    if b.wings == 29 && glide_draw(app, 3) == 0 {
        let idx = glide_spawn(app, &b, 8, -40, -15.0, 30, 30, 6, 100, 2.4, Color::default());
        let fade = glide_draw(app, 10) == 0;
        let d = app.dust.slot_mut(idx);
        d.no_gravity = true;
        d.velocity *= 0.3;
        if fade {
            d.fade_in = 2.0;
        }
    }
    // `:27122-27148`: wings 6 (the `Next(10)` gate) / else wings 5 (the `Next(6)`
    // gate) — the else-if pair.
    if b.wings == 6 {
        if glide_draw(app, 10) == 0 {
            glide_puff(app, &b, 4, -40, -12.0, 30, 20, 55, 200, 1.0, Color::default(), GlidePost::Plain03);
        }
    } else if b.wings == 5 && glide_draw(app, 6) == 0 {
        glide_puff(app, &b, 6, -30, 0.0, 18, b.height as i32, 58, 255, 1.2, Color::default(), GlidePost::Plain03);
    }
    // `:27149`: wings 4 — the rocket-boots glide puff (the wingFrame machine +
    // Item13 cadence are cited seams). The `Next(2)` position pick draws always.
    if b.wings == 4 {
        let mut x3 = b.x + b.width * 0.5 + 16.0;
        if b.facing > 0 {
            x3 = b.x + b.width * 0.5 - 26.0;
        }
        let mut y3 = b.y + b.height - 18.0;
        if glide_draw(app, 2) == 1 {
            x3 = b.x + b.width * 0.5 + 8.0;
            if b.facing > 0 {
                x3 = b.x + b.width * 0.5 - 20.0;
            }
            y3 += 6.0;
        }
        let idx = nd(app, Vec2::new(x3, y3), 8, 8, 6, 0.0, 0.0, 100, Color::default(), 1.5);
        let d = app.dust.slot_mut(idx);
        d.velocity.x *= 0.3;
        d.velocity.y += 10.0;
        d.no_gravity = true;
    }
    // `:27240`: wings 26 — one type-217 puff + the `Next(2)` second puff.
    if b.wings == 26 {
        glide_217(app, &b, 6, -30, 217, 18, b.height as i32, 100, 1.4);
        if glide_draw(app, 2) == 0 {
            glide_217(app, &b, -24, 12, 217, 12, (b.height / 2.0) as i32, 100, 1.4);
        }
    }
    // `:27274`: wings 37 — the colour `NextFloat` + the `SelectRandom(31,31,31)`
    // roll, one type-31 puff + the `Next(3)` second puff.
    if b.wings == 37 {
        let t = glide_draw_float(app);
        // `Color.Lerp(Black, White, t)` — the XNA byte multiply rounds (`(byte)(v*t+0.5)`).
        let c = (255.0 * t + 0.5) as u8;
        let color = Color::new(c, c, c, 255);
        let _ty = [31, 31, 31][glide_draw(app, 3) as usize]; // the SelectRandom roll
        glide_217(app, &b, 6, -30, 31, 24, b.height as i32, 100, 0.7);
        let _ = color; // the type-55 colour arm is dead (the type is always 31)
        if glide_draw(app, 3) == 0 {
            let _ty2 = [31, 31, 31][glide_draw(app, 3) as usize];
            glide_217(app, &b, -24, 12, 31, 12, (b.height / 2.0) as i32, 140, 0.7);
        }
    }
}

/// One `UpdatePlayers` draw for the wing-glide gates (`Main.rand`), taken
/// without holding the stream borrow across an `app` mutation.
fn glide_draw(app: &mut App, n: i32) -> i32 {
    app.rngs
        .stream(crate::app::RngKey::UpdatePlayers)
        .next_max(n)
}

/// One `UpdatePlayers` float draw (the wings-37 colour lerp).
fn glide_draw_float(app: &mut App) -> f32 {
    app.rngs
        .stream(crate::app::RngKey::UpdatePlayers)
        .next_float()
}

/// The wing-glide post-arm shapes.
#[derive(Clone, Copy)]
enum GlidePost {
    /// `fadeIn = 1.1`, `noGravity`, `noLight`, `velocity *= 0.3`.
    Fade03,
    /// `noGravity`, `velocity *= 0.3`.
    NoGrav03,
    /// `velocity *= 0.3`.
    Plain03,
}

/// Spawn one wing-glide puff at the facing-mirrored offset and apply the post-arm.
#[allow(clippy::too_many_arguments)]
fn glide_puff(app: &mut App, b: &WingGlideDust, off_l: i32, off_r: i32, dy: f32, w: i32, h: i32, ty: i32, alpha: i32, scale: f32, color: Color, post: GlidePost) {
    let idx = glide_spawn(app, b, off_l, off_r, dy, w, h, ty, alpha, scale, color);
    let d = app.dust.slot_mut(idx);
    match post {
        GlidePost::Fade03 => {
            d.fade_in = 1.1;
            d.no_gravity = true;
            d.no_light = true;
            d.velocity *= 0.3;
        }
        GlidePost::NoGrav03 => {
            d.no_gravity = true;
            d.velocity *= 0.3;
        }
        GlidePost::Plain03 => {
            d.velocity *= 0.3;
        }
    }
}

/// Spawn one wing-glide puff at `(x + width/2 + offset, y + height/2 + dy)`,
/// the offset mirrored by facing (`direction == 1` takes the right value).
#[allow(clippy::too_many_arguments)]
fn glide_spawn(app: &mut App, b: &WingGlideDust, off_l: i32, off_r: i32, dy: f32, w: i32, h: i32, ty: i32, alpha: i32, scale: f32, color: Color) -> usize {
    let off = if b.facing == 1 { off_r } else { off_l };
    let pos = Vec2::new(b.x + b.width * 0.5 + off as f32, b.y + b.height * 0.5 + dy);
    nd(app, pos, w, h, ty, 0.0, 0.0, alpha, color, scale)
}

/// The wings-26/37 type-217/31 puff at `(x + width/2 + offset, y)` (the second
/// puff's `y + height/2` gravDir fold collapses to `y`), `noGravity`/`noLight`,
/// `velocity /= 4` then `- base.velocity`.
#[allow(clippy::too_many_arguments)]
fn glide_217(app: &mut App, b: &WingGlideDust, off_l: i32, off_r: i32, ty: i32, w: i32, h: i32, alpha: i32, scale: f32) {
    let off = if b.facing == 1 { off_r } else { off_l };
    let pos = Vec2::new(b.x + b.width * 0.5 + off as f32, b.y);
    let idx = nd(app, pos, w, h, ty, 0.0, 0.0, alpha, Color::default(), scale);
    let d = app.dust.slot_mut(idx);
    d.no_gravity = true;
    d.no_light = true;
    d.velocity /= 4.0;
    d.velocity -= Vec2::new(b.vx, b.vy);
}

// ---- W4-f3: the liquid entry/exit splash dust (`Player.cs:27783-27940`) -----

/// Drain the liquid entry/exit splash dust burst (`Player.cs:27783-27940`). The
/// C# runs the dust loop BEFORE `PlaySound(19)`, so the drain precedes the splash
/// sound on the `UpdatePlayers` stream. The shimmer arms roll a `Next(6)` colour
/// pick per puff; the water arms use `Dust.dustWater()` (the biome water style).
pub(crate) fn drain_splash_dust(app: &mut App, b: SplashDust) {
    match b.arm {
        SplashArm::ShimmerEntry => splash_shimmer(app, &b, 0.8),
        SplashArm::ShimmerExit => splash_shimmer(app, &b, 0.75),
        SplashArm::HoneyEntry | SplashArm::HoneyExit => splash_simple(app, &b, 152, 20, -8.0, -1.0, 1.3, true),
        SplashArm::LavaEntry | SplashArm::LavaExit => splash_simple(app, &b, 35, 20, -8.0, -1.5, 1.3, true),
        SplashArm::WaterEntry => {
            let ty = tr_core::particles::dust_water(b.water_style);
            splash_simple(app, &b, ty, 50, -8.0, -3.0, 0.8, true);
        }
        SplashArm::WaterExit => {
            let ty = tr_core::particles::dust_water(b.water_style);
            splash_simple(app, &b, ty, 50, 0.0, -4.0, 0.8, true);
        }
    }
}

/// The shimmer splash (`:27783-27805` entry / `:27877-27899` exit): 50 type-308
/// puffs at `(x-6, y+height/2)`, box `(width+12)×24`, `velocity.Y -= 4`,
/// `velocity.X *= 2.5`, scale 0.8 (entry) / 0.75 (exit), noGravity, then a
/// `Next(6)` colour pick (0 → (255,255,210), 1 → (190,245,255), 2 → (255,150,255),
/// else (190,175,255)).
fn splash_shimmer(app: &mut App, b: &SplashDust, scale: f32) {
    // `:27785`: `(float)(base.height / 2)` — INTEGER half (the wall-corner
    // int-division class). f3's float half drifted 0.5 px on odd heights;
    // e1's liquid_splash already uses the int form. KEEP-E1 (W4-f3 seal).
    let pos = Vec2::new(b.x - 6.0, b.y + (b.height as i32 / 2) as f32);
    for _ in 0..50 {
        let idx = nd(app, pos, b.width as i32 + 12, 24, 308, 0.0, 0.0, 0, Color::default(), 1.0);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let d = app.dust.slot_mut(idx);
        d.velocity.y -= 4.0;
        d.velocity.x *= 2.5;
        d.scale = scale;
        d.no_gravity = true;
        d.color = match rng.next_max(6) {
            0 => Color::new(255, 255, 210, 255),
            1 => Color::new(190, 245, 255, 255),
            2 => Color::new(255, 150, 255, 255),
            _ => Color::new(190, 175, 255, 255),
        };
    }
}

/// The honey/lava/water splash (`:27810-27846` / `:27904-27940`): `count` puffs
/// of `ty` at `(x-6, y+height/2+dy)`, box `(width+12)×24`, `velocity.Y += y_sub`,
/// `velocity.X *= 2.5`, scale set, alpha 100 (when `alpha100`), noGravity.
#[allow(clippy::too_many_arguments)]
fn splash_simple(app: &mut App, b: &SplashDust, ty: i32, count: i32, dy: f32, y_sub: f32, scale: f32, alpha100: bool) {
    // `:27812`/`:27825`/`:27839` et al.: `(float)(base.height / 2)` — INTEGER
    // half (the wall-corner class). KEEP-E1's int form (W4-f3 seal).
    let pos = Vec2::new(b.x - 6.0, b.y + (b.height as i32 / 2) as f32 + dy);
    for _ in 0..count {
        let idx = nd(app, pos, b.width as i32 + 12, 24, ty, 0.0, 0.0, 0, Color::default(), 1.0);
        let d = app.dust.slot_mut(idx);
        d.velocity.y += y_sub;
        d.velocity.x *= 2.5;
        d.scale = scale;
        if alpha100 {
            d.alpha = 100;
        }
        d.no_gravity = true;
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

    // ---- the DashMovement lane (`Player.cs:21259-21512`) ------------------

    fn dash_burst(dash: i32, active: bool) -> DashDust {
        DashDust {
            dash,
            active,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 16.9,
            vy: 0.0,
            facing: 1,
            mount: -1,
        }
    }

    /// The tick-side queue: the double-tap start snapshots AFTER the launch
    /// writes (`vx = 16.9*dir`, `Player.cs:21411`), the next tick's trail
    /// queues `active`; the dash-2 loops are dead — no queue either tick.
    #[test]
    fn w4_d51_the_dash_ticks_queue_start_then_trail() {
        let w = World::new("t".into(), 64, 128);
        let tap = |p: &mut Player, right: bool| {
            p.update(&w, crate::player::PlayerInput { right, ..Default::default() });
        };
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            tap(&mut p, false);
        }
        p.pinned_dash = 1;
        tap(&mut p, true);
        tap(&mut p, false);
        tap(&mut p, true);
        let b = p.take_dash_dust().expect("the Tabi start queues its burst");
        assert_eq!((b.dash, b.active), (1, false));
        assert_eq!(b.vx, 16.9, "the snapshot rides the launch write");
        assert_eq!(b.mount, -1);
        tap(&mut p, false);
        let b = p.take_dash_dust().expect("the active tick queues the trail");
        assert_eq!((b.dash, b.active), (1, true));
        // The shield's start/trail loops are `m < 0` — dead.
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            tap(&mut p, false);
        }
        p.pinned_dash = 2;
        tap(&mut p, true);
        tap(&mut p, false);
        tap(&mut p, true);
        assert!(p.take_dash_dust().is_none(), "the dash-2 start is dead");
        assert_eq!(p.eoc_dash, 15, "the grace still arms");
        tap(&mut p, false);
        assert!(p.take_dash_dust().is_none(), "the dash-2 trail is dead");
    }

    /// `:21259-21270` — the Tabi trail: 2 type-31 puffs, the airborne/grounded
    /// box fold (`num18 = height/42`), `velocity *= 0.1` + the scale roll.
    #[test]
    fn w4_d51_the_tabi_trail_folds_the_box_by_vy() {
        for vy in [0.0f32, 3.0] {
            let mut app = rig(0x5EED);
            let mut b = dash_burst(1, true);
            b.vy = vy;
            drain_dash_dust(&mut app, b);
            let slots = live(&app);
            assert_eq!(slots.len(), 2);
            let mut rng = UnifiedRandom::new(0x5EED);
            let num19 = 4.0 * (b.height / 42.0);
            let num20 = (16.0 * (b.height / 42.0)) as i32;
            let (py, h) = if vy != 0.0 {
                (b.y + b.height / 2.0 - num19 * 2.0, num20)
            } else {
                (b.y + b.height - num19, num20 / 2)
            };
            for d in &slots {
                assert_eq!(d.dust_type, 31);
                assert_eq!(d.alpha, 100);
                let px = b.x + rng.next_max(16) as f32 + 4.0;
                let py2 = py + rng.next_max((h - 4).max(1)) as f32 + 4.0;
                assert_eq!(d.position, Vec2::new(px, py2));
                let vx = rng.next_range(-20, 21) as f32 * 0.1;
                let vyy = rng.next_range(-20, 21) as f32 * 0.1;
                let _ = rng.next_max(3);
                let scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * 1.4;
                assert_eq!(d.velocity, Vec2::new(vx, vyy) * 0.1);
                assert_eq!(d.scale, scale * (1.0 + rng.next_max(20) as f32 * 0.01));
            }
            assert_eq!(stream_of(&mut app), rng);
            assert_eq!(stream_of(&mut app), draws_after(0x5EED, 2 * 7));
        }
    }

    /// `:21286-21302` — the Solar trail: 4 type-6 puffs (the tweak draw rides
    /// `new_dust`), the scale roll, `noGravity`, the `Next(2)` fadeIn gate.
    #[test]
    fn w4_d51_the_solar_trail_gates_the_fade() {
        let mut app = rig(0x5EED);
        let b = dash_burst(3, true);
        drain_dash_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 4);
        let mut rng = UnifiedRandom::new(0x5EED);
        let num25 = 4.0 * (b.height / 42.0);
        for d in &slots {
            assert_eq!(d.dust_type, 6);
            let px = b.x + rng.next_max(16) as f32 + 4.0;
            let py = b.y + num25 + rng.next_max(34 - 4) as f32 + 4.0;
            assert_eq!(d.position, Vec2::new(px, py));
            let mut vx = rng.next_range(-20, 21) as f32 * 0.1;
            let _ = rng.next_range(-20, 21);
            let _ = rng.next_max(3);
            let scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * 1.7;
            let tweak_y = rng.next_range(-10, 6) as f32 * 0.1;
            vx *= 0.3;
            assert_eq!(d.velocity, Vec2::new(vx, tweak_y) * 0.1);
            assert_eq!(d.scale, scale * 0.7 * (1.0 + rng.next_max(20) as f32 * 0.01));
            assert!(d.no_gravity);
            let fade = if rng.next_max(2) == 0 { 0.5 } else { 0.0 };
            assert_eq!(d.fade_in, fade);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 4 * 9));
    }

    /// `:21328-21343` — the Vortex trail: the per-puff `{68,69,70}` roll, the
    /// Tabi box fold, `velocity *= 0.2`, the scale + fadeIn rolls.
    #[test]
    fn w4_d51_the_vortex_trail_rolls_the_type_per_puff() {
        let mut app = rig(0x5EED);
        let mut b = dash_burst(5, true);
        b.vy = 2.0;
        drain_dash_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2);
        let mut rng = UnifiedRandom::new(0x5EED);
        let num32 = 4.0 * (b.height / 42.0);
        let num33 = (16.0 * (b.height / 42.0)) as i32;
        for d in &slots {
            let ty = [68, 69, 70][rng.next_max(3) as usize];
            assert_eq!(d.dust_type, ty);
            let px = b.x + rng.next_max(16) as f32 + 4.0;
            let py = b.y + b.height / 2.0 - num32 * 2.0 + rng.next_max(num33 - 4) as f32 + 4.0;
            assert_eq!(d.position, Vec2::new(px, py));
            let vx = rng.next_range(-20, 21) as f32 * 0.1;
            let vyy = rng.next_range(-20, 21) as f32 * 0.1;
            let _ = rng.next_max(3);
            let scale = 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
            assert_eq!(d.velocity, Vec2::new(vx, vyy) * 0.2);
            assert_eq!(d.scale, scale * (1.0 + rng.next_max(20) as f32 * 0.01));
            assert_eq!(d.fade_in, 0.5 + rng.next_max(20) as f32 * 0.01);
            assert!(d.no_gravity);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 2 * 9));
    }

    /// `:21344-21372` — the Chillet trail: the centre light (no draws), 8
    /// type-306 puffs at `(x + facing*80, y + 12)`, the colour cube-lerp; the
    /// warm base on mounts 63/65.
    #[test]
    fn w4_d51_the_chillet_trail_lights_and_cube_lerps() {
        for (mount, warm) in [(-1i32, false), (63, true), (65, true)] {
            let mut app = rig(0x5EED);
            let mut b = dash_burst(6, true);
            b.mount = mount;
            drain_dash_dust(&mut app, b);
            app.entity_lights.end_frame();
            let lights = app.entity_lights.visible();
            assert_eq!(lights.len(), 1);
            let rgb = if warm { [0.6, 0.3, 0.1] } else { [0.5, 0.1, 0.6] };
            assert_eq!((lights[0].x, lights[0].y, lights[0].rgb), (19, 38, rgb));
            let slots = live(&app);
            assert_eq!(slots.len(), 8);
            let base = if warm {
                Color::new(255, 178, 127, 255)
            } else {
                Color::new(229, 178, 255, 255)
            };
            let mut rng = UnifiedRandom::new(0x5EED);
            for d in &slots {
                assert_eq!(d.dust_type, 306);
                let px = b.x + 80.0 + rng.next_max(16) as f32 + 4.0;
                let py = b.y + 12.0 + rng.next_max(22 - 4) as f32 + 4.0;
                assert_eq!(d.position, Vec2::new(px, py));
                let vx = rng.next_range(-20, 21) as f32 * 0.1;
                let vyy = rng.next_range(-20, 21) as f32 * 0.1;
                let _ = rng.next_max(3);
                let scale = 1.0 + rng.next_range(-20, 21) as f32 * 0.01;
                assert_eq!(d.velocity, Vec2::new(vx, vyy) * 1.5 + Vec2::new(1.5, 0.0));
                assert_eq!(d.scale, scale * (1.0 + rng.next_max(40) as f32 * 0.01));
                assert_eq!(d.fade_in, 0.5 + rng.next_max(20) as f32 * 0.01);
                assert!(d.no_gravity);
                let t = rng.next_float();
                assert_eq!(d.color, lerp_color(base, Color::WHITE, t * t * t));
            }
            assert_eq!(stream_of(&mut app), rng);
            assert_eq!(stream_of(&mut app), draws_after(0x5EED, 8 * 9));
        }
    }

    /// `DelegateMethods.cs:150-188` — the Bat/Pixie delegate: even iterations
    /// deactivate (no draws), odd ones re-roll the hitbox position (two
    /// NextFloat draws) and shrink ×0.75.
    #[test]
    fn w4_d51_the_bat_delegate_halves_the_trail() {
        for mount in [56, 61] {
            let mut app = rig(0x5EED);
            let mut b = dash_burst(1, true);
            b.mount = mount;
            drain_dash_dust(&mut app, b);
            let slots = live(&app);
            assert_eq!(slots.len(), 1, "the even puff deactivates");
            assert_eq!(app.dust.slot(0).active, false);
            let mut rng = UnifiedRandom::new(0x5EED);
            // Puff 0: six default draws + the scale roll, then deactivated.
            for _ in 0..6 {
                rng.next();
            }
            let _ = rng.next_max(20);
            // Puff 1: six default + scale roll + the two hitbox floats.
            let d = slots[0];
            let _ = rng.next_max(16);
            let _ = rng.next_max(4);
            let vx = rng.next_range(-20, 21) as f32 * 0.1;
            let vyy = rng.next_range(-20, 21) as f32 * 0.1;
            let _ = rng.next_max(3);
            let scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * 1.4;
            assert_eq!(d.velocity, Vec2::new(vx, vyy) * 0.1);
            let scale = scale * (1.0 + rng.next_max(20) as f32 * 0.01);
            let px = b.x + rng.next_float() * b.width;
            let py = b.y + rng.next_float() * b.height;
            assert_eq!(d.position, Vec2::new(px, py));
            assert_eq!(d.scale, scale * 0.75);
            assert_eq!(stream_of(&mut app), rng);
        }
    }

    /// `:21419-21426` — the Tabi start: 20 type-31 puffs, the ±5 jitter, the
    /// ×0.2 velocity and the scale roll (9 draws each).
    #[test]
    fn w4_d51_the_tabi_start_jitters_twenty_puffs() {
        let mut app = rig(0x5EED);
        let b = dash_burst(1, false);
        drain_dash_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 20);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 31);
            let mut px = b.x + rng.next_max(16) as f32 + 4.0;
            let mut py = b.y + rng.next_max(38) as f32 + 4.0;
            let vx = rng.next_range(-20, 21) as f32 * 0.1;
            let vyy = rng.next_range(-20, 21) as f32 * 0.1;
            let _ = rng.next_max(3);
            let scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * 2.0;
            px += rng.next_range(-5, 6) as f32;
            py += rng.next_range(-5, 6) as f32;
            assert_eq!(d.position, Vec2::new(px, py));
            assert_eq!(d.velocity, Vec2::new(vx, vyy) * 0.2);
            assert_eq!(d.scale, scale * (1.0 + rng.next_max(20) as f32 * 0.01));
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 20 * 9));
    }

    /// `:21474-21484` — the Solar start: the Tabi shape on type 6 (the tweak
    /// draw included), plus `noGravity` and the flat fadeIn 0.5.
    #[test]
    fn w4_d51_the_solar_start_fades_flat() {
        let mut app = rig(0x5EED);
        let b = dash_burst(3, false);
        drain_dash_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 20);
        for d in &slots {
            assert_eq!(d.dust_type, 6);
            assert!(d.no_gravity);
            assert_eq!(d.fade_in, 0.5);
        }
        // 6 default + 1 tweak + 2 jitter + 1 scale per puff.
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 20 * 10));
    }

    /// `:21500-21510` — the Vortex start: the per-puff type roll, the jitter,
    /// then the velocity REPLACED by the centre-relative direction ×2.
    #[test]
    fn w4_d51_the_vortex_start_aims_from_the_centre() {
        let mut app = rig(0x5EED);
        let b = dash_burst(5, false);
        drain_dash_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 20);
        let center = Vec2::new(b.x + b.width * 0.5, b.y + b.height * 0.5);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            let ty = [68, 69, 70][rng.next_max(3) as usize];
            assert_eq!(d.dust_type, ty);
            let mut px = b.x + rng.next_max(16) as f32 + 4.0;
            let mut py = b.y + rng.next_max(38) as f32 + 4.0;
            let _ = rng.next_range(-20, 21);
            let _ = rng.next_range(-20, 21);
            let _ = rng.next_max(3);
            let scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * 1.5;
            px += rng.next_range(-5, 6) as f32;
            py += rng.next_range(-5, 6) as f32;
            assert_eq!(d.position, Vec2::new(px, py));
            let delta = Vec2::new(px, py) - center;
            assert_eq!(d.velocity, delta.normalize() * 2.0);
            assert_eq!(d.scale, scale * (1.0 + rng.next_max(20) as f32 * 0.01));
            assert_eq!(d.fade_in, 0.5 + rng.next_max(20) as f32 * 0.01);
            assert!(d.no_gravity);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 20 * 11));
    }

    // ---- the fast-run trail (`Player.cs:20217-20298`) ---------------------

    fn run_burst(arm: FastRunArm) -> FastRunDust {
        FastRunDust {
            arm,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 6.5,
            vy: 0.0,
            gfx_off_y: 2.0,
        }
    }

    /// The tick-side arm picker (`Player.cs:20217-20298`): the else-if chain
    /// in source order — wings-3/rocket-6 first, then sail, desert, cold,
    /// fairy, hellfire, plain.
    #[test]
    fn w4_d51_the_fast_run_picker_follows_the_chain() {
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        // The picker is private; drive it through the flags + a fast-run
        // tick is heavy — assert the grant chain instead: each boot arms its
        // own visual flag through `apply_equip_functional`'s per-slot pass.
        p.equipped[3] = 3200; // Sailfish
        p.update(&w, crate::player::PlayerInput::default());
        assert!(p.sail_dash, "the Sailfish arms sailDash");
        assert!(!p.cold_dash && !p.desert_dash);
        p.equipped[3] = 1579; // Flurry
        p.update(&w, crate::player::PlayerInput::default());
        assert!(p.cold_dash, "the Flurry arms coldDash (and cancels sail)");
        assert!(!p.sail_dash);
        p.equipped[3] = 4874; // Hellfire Treads
        p.update(&w, crate::player::PlayerInput::default());
        assert!(p.hellfire_treads && p.vanity_rocket_boots == 5);
        assert!(!p.cold_dash);
        p.equipped[3] = 3993; // Fairy
        p.update(&w, crate::player::PlayerInput::default());
        assert!(p.fairy_boots && p.vanity_rocket_boots == 2);
        assert!(!p.hellfire_treads);
        p.equipped[3] = 4055; // Sand Boots
        p.update(&w, crate::player::PlayerInput::default());
        assert!(p.desert_dash && !p.fairy_boots);
        // Unequip: ResetEffects clears them all.
        p.equipped[3] = 0;
        p.update(&w, crate::player::PlayerInput::default());
        assert!(!p.desert_dash && p.vanity_rocket_boots == 0);
    }

    /// `:20217-20225` — wings-3/rocket-6: two type-186 puffs, ×0.025 then
    /// ×0.2 velocity folds.
    #[test]
    fn w4_d51_the_wings_rocket_trail_folds_twice() {
        let mut app = rig(0x5EED);
        let b = run_burst(FastRunArm::WingsRocket);
        drain_fast_run(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2);
        let mut rng = UnifiedRandom::new(0x5EED);
        for (d, fold) in slots.iter().zip([0.025f32, 0.2]) {
            assert_eq!(d.dust_type, 186);
            assert_eq!(d.alpha, 50);
            let px = b.x - 4.0 + rng.next_max(28 - 4) as f32 + 4.0;
            let py = b.y + b.height + rng.next_max(1) as f32 + 4.0;
            assert_eq!(d.position, Vec2::new(px, py));
            let vx = rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5;
            let vyy = rng.next_range(-20, 21) as f32 * 0.1 + b.vy * 0.5;
            let _ = rng.next_max(3);
            let _ = rng.next_range(-20, 21);
            assert_eq!(d.velocity, Vec2::new(vx, vyy) * fold);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 2 * 6));
    }

    /// `:20226-20240` — sailDash: four type-253 puffs, the ×0.2 folds, the
    /// scale roll and the three dead `vector` draws.
    #[test]
    fn w4_d51_the_sail_trail_keeps_the_dead_draws() {
        let mut app = rig(0x5EED);
        let b = run_burst(FastRunArm::Sail);
        drain_fast_run(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 4);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 253);
            assert!(d.no_gravity);
            let px = b.x - 4.0 + rng.next_max(24) as f32 + 4.0;
            let py = b.y + rng.next_max(38) as f32 + 4.0;
            assert_eq!(d.position, Vec2::new(px, py));
            let vx = (rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5) * 0.2;
            let vyy = (rng.next_range(-20, 21) as f32 * 0.1 + b.vy * 0.5) * 0.2;
            let _ = rng.next_max(3);
            let scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * 1.5;
            assert_eq!(d.velocity, Vec2::new(vx, vyy));
            assert_eq!(d.scale, scale + rng.next_range(-5, 3) as f32 * 0.1);
            let _ = rng.next_range(-100, 101);
            let _ = rng.next_range(-100, 101);
            let _ = rng.next_max(81);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 4 * 10));
    }

    /// `:20241-20247` — desertDash: one type-32 Direct puff, ×0.2 then Y−2.
    #[test]
    fn w4_d51_the_desert_trail_kicks_up() {
        let mut app = rig(0x5EED);
        let b = run_burst(FastRunArm::Desert);
        drain_fast_run(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let mut rng = UnifiedRandom::new(0x5EED);
        let d = slots[0];
        assert_eq!(d.dust_type, 32);
        let _ = rng.next_max(24);
        let _ = rng.next_max(1);
        let vx = (rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5) * 0.2;
        let vyy = (rng.next_range(-20, 21) as f32 * 0.1 + b.vy * 0.5) * 0.2 - 2.0;
        let _ = rng.next_max(3);
        let _ = rng.next_range(-20, 21);
        assert_eq!(d.velocity, Vec2::new(vx, vyy));
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 6));
    }

    /// `:20248-20260` — coldDash: two type-76 puffs on the gfxOffY foot line,
    /// the left/right box split, the scale roll and the ×0.001/Y−0.003 fold.
    #[test]
    fn w4_d51_the_cold_trail_splits_the_box() {
        let mut app = rig(0x5EED);
        let b = run_burst(FastRunArm::Cold);
        drain_fast_run(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2);
        let mut rng = UnifiedRandom::new(0x5EED);
        for (j, d) in slots.iter().enumerate() {
            assert_eq!(d.dust_type, 76);
            assert_eq!(d.alpha, 0);
            assert!(d.no_gravity && d.no_light);
            let px0 = if j != 0 { b.x + b.width / 2.0 } else { b.x };
            let px = px0 + rng.next_max(10 - 4) as f32 + 4.0;
            let py = b.y + b.height + b.gfx_off_y + rng.next_max(2) as f32 + 4.0;
            assert_eq!(d.position, Vec2::new(px, py));
            let vx = rng.next_range(-20, 21) as f32 * 0.1;
            let vyy = rng.next_range(-20, 21) as f32 * 0.1;
            let _ = rng.next_max(3);
            let scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * 1.35;
            assert_eq!(d.scale, scale * (1.0 + rng.next_range(20, 40) as f32 * 0.01));
            assert_eq!(d.velocity, Vec2::new(vx, vyy) * 0.001 - Vec2::new(0.0, 0.003));
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 2 * 7));
    }

    /// `:20261-20282` — fairyBoots: ONE type roll before the loop, two puffs
    /// (scale 1.5 then 1.0), the k=2 position ride. The rolled type may hit
    /// the tweak chain (242/64/63) — the mirror walks it conditionally.
    #[test]
    fn w4_d51_the_fairy_trail_rolls_one_type() {
        let mut app = rig(0x5EED);
        let b = run_burst(FastRunArm::Fairy);
        drain_fast_run(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2);
        let mut rng = UnifiedRandom::new(0x5EED);
        let ty = [61, 61, 61, 242, 64, 63][rng.next_max(6) as usize];
        // Every fairy type hits the tweak chain: 61/63/64 in `59..=65`, 242
        // listed (`Dust.cs:214-220`).
        let tweaked = matches!(ty, 61 | 242 | 64 | 63);
        for (k, d) in slots.iter().enumerate() {
            assert_eq!(d.dust_type, ty);
            let scale_arg = if k == 1 { 1.0 } else { 1.5 };
            let px = b.x - 4.0 + rng.next_max(24) as f32 + 4.0;
            let py = b.y + b.height + rng.next_max(1) as f32 + 4.0;
            let mut vx = rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5;
            let mut vyy = rng.next_range(-20, 21) as f32 * 0.1 + b.vy * 0.5;
            let _ = rng.next_max(3);
            let mut scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * scale_arg;
            if tweaked {
                vyy = rng.next_range(-10, 6) as f32 * 0.1;
                vx *= 0.3;
                scale *= 0.7;
            }
            let vel = Vec2::new(vx, vyy) * 1.5;
            assert_eq!(d.velocity, vel);
            let pos = Vec2::new(px, py) + if k == 1 { vel } else { Vec2::ZERO };
            assert_eq!(d.position, pos);
            assert_eq!(d.scale, scale);
            assert!(d.no_gravity && d.no_light_emittance);
        }
        assert_eq!(stream_of(&mut app), rng);
        let n = 1 + 2 * 6 + if tweaked { 2 } else { 0 };
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, n));
    }

    /// `:20283-20291` — hellfireTreads: one type-6 puff (the tweak draw
    /// included), X ×0.2, Y replaced by the −1.5−NextFloat*0.5 rise.
    #[test]
    fn w4_d51_the_hellfire_trail_rises() {
        let mut app = rig(0x5EED);
        let b = run_burst(FastRunArm::Hellfire);
        drain_fast_run(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let mut rng = UnifiedRandom::new(0x5EED);
        let d = slots[0];
        assert_eq!(d.dust_type, 6);
        assert_eq!(d.fade_in, 0.5);
        assert!(d.no_gravity);
        let _ = rng.next_max(24);
        let _ = rng.next_max(1);
        let mut vx = rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5;
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_max(3);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-10, 6); // the type-6 tweak draw
        vx *= 0.3;
        assert_eq!(d.velocity.x, vx * 0.2);
        assert_eq!(d.velocity.y, -1.5 - rng.next_float() * 0.5);
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 8));
    }

    /// `:20292-20298` — the plain foot puff: one type-16, ×0.2 per axis.
    #[test]
    fn w4_d51_the_plain_trail_damps() {
        let mut app = rig(0x5EED);
        let b = run_burst(FastRunArm::Plain);
        drain_fast_run(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let mut rng = UnifiedRandom::new(0x5EED);
        let d = slots[0];
        assert_eq!(d.dust_type, 16);
        assert_eq!(d.alpha, 50);
        let _ = rng.next_max(24);
        let _ = rng.next_max(1);
        let vx = (rng.next_range(-20, 21) as f32 * 0.1 - b.vx * 0.5) * 0.2;
        let vyy = (rng.next_range(-20, 21) as f32 * 0.1 + b.vy * 0.5) * 0.2;
        let _ = rng.next_max(3);
        let _ = rng.next_range(-20, 21);
        assert_eq!(d.velocity, Vec2::new(vx, vyy));
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 6));
    }

    // ---- W4-f3: DoubleJumpVisuals / stoned / drowning-bubble witnesses ------

    fn dj_burst(kind: ExtraJump) -> DoubleJumpDust {
        DoubleJumpDust {
            kind,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 3.0,
            vy: -5.01,
            jump: 30,
            sandstorm_gore_gate: false,
        }
    }

    /// The six default `NewDust` draws (`Dust.cs:153-193`): position X/Y,
    /// velocity X/Y, frame row, scale. Returns the rolled velocity + scale.
    fn default_six(rng: &mut UnifiedRandom, w: i32, h: i32, sx: f32, sy: f32, scale_arg: f32) -> (Vec2, f32) {
        let _ = rng.next_max(w.max(5) - 4);
        let _ = rng.next_max(h.max(5) - 4);
        let vx = rng.next_range(-20, 21) as f32 * 0.1 + sx;
        let vy = rng.next_range(-20, 21) as f32 * 0.1 + sy;
        let _ = rng.next_max(3);
        let scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * scale_arg;
        (Vec2::new(vx, vy), scale)
    }

    /// `:21894-21904` — the Cloud visual: one type-16, the Fart-style velocity
    /// post-arms, 6 draws.
    #[test]
    fn w4_f3_the_cloud_visual_mirrors_the_fart_shape() {
        let mut app = rig(0x5EED);
        let b = dj_burst(ExtraJump::Cloud);
        drain_double_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let mut rng = UnifiedRandom::new(0x5EED);
        let d = slots[0];
        assert_eq!(d.dust_type, 16);
        assert_eq!(d.alpha, 100);
        let (v, _) = default_six(&mut rng, b.width as i32 + 8, 4, -b.vx * 0.5, b.vy * 0.5, 1.5);
        assert_eq!(d.velocity.x, v.x * 0.5 - b.vx * 0.1);
        assert_eq!(d.velocity.y, v.y * 0.5 - b.vy * 0.3);
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 6));
    }

    /// `:21905-21918` — the Sandstorm visual: 3 type-124 puffs, scale arg
    /// `num4 = (jump/75+1)/2`, post `velocity *= 0.5*num4` / `fadeIn = 1.5*num4`.
    #[test]
    fn w4_f3_the_sandstorm_visual_scales_by_the_jump_counter() {
        let mut app = rig(0x5EED);
        let b = dj_burst(ExtraJump::Sandstorm);
        drain_double_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 3);
        let num4 = (b.jump as f32 / 75.0 + 1.0) / 2.0;
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 124);
            assert_eq!(d.alpha, 150);
            let (v, _) = default_six(&mut rng, b.width as i32, 32, b.vx * 0.3, b.vy * 0.3, num4);
            assert_eq!(d.velocity, v * (0.5 * num4));
            assert_eq!(d.fade_in, 1.5 * num4);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 3 * 6));
    }

    /// W4-f3 seal (KEEP-E1) — `:21920-21922`: when the `miscCounter % 3 == 0`
    /// gate fires, the Sandstorm gore sibling's type ARGUMENT `Next(220,223)`
    /// draws on `UpdatePlayers` AFTER the three puffs' 18 default draws (the
    /// gore body is d55's, unported; the argument draw is NOT — it must ride
    /// the stream). The dust records are unchanged; the stream advances one
    /// extra `next_range` step.
    #[test]
    fn w4_f3_the_sandstorm_gore_gate_draws_the_argument() {
        let mut app = rig(0x5EED);
        let mut b = dj_burst(ExtraJump::Sandstorm);
        b.sandstorm_gore_gate = true;
        drain_double_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 3, "the dust chain is unchanged");
        let num4 = (b.jump as f32 / 75.0 + 1.0) / 2.0;
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            let (v, _) = default_six(&mut rng, b.width as i32, 32, b.vx * 0.3, b.vy * 0.3, num4);
            assert_eq!(d.velocity, v * (0.5 * num4));
            assert_eq!(d.fade_in, 1.5 * num4);
        }
        // The gore argument draw rides after the 18 default draws.
        let _ = rng.next_range(220, 223);
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:21927-21938` — the Fart visual: the Cloud shape plus `velocity *= 0.5`.
    #[test]
    fn w4_f3_the_fart_visual_halves_again() {
        let mut app = rig(0x5EED);
        let b = dj_burst(ExtraJump::Fart);
        drain_double_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let mut rng = UnifiedRandom::new(0x5EED);
        let d = slots[0];
        assert_eq!(d.dust_type, 188);
        let (v, _) = default_six(&mut rng, b.width as i32 + 8, 4, -b.vx * 0.5, b.vy * 0.5, 1.5);
        let post = Vec2::new(v.x * 0.5 - b.vx * 0.1, v.y * 0.5 - b.vy * 0.3) * 0.5;
        assert_eq!(d.velocity, post);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 6));
    }

    /// `:21939-21947` — the Unicorn visual: one rolled type (`Next(3)` before
    /// the internals), zero velocity, `scale = 0.5 + NextFloat*0.8`,
    /// `fadeIn = 1 + NextFloat*2`. 9 draws.
    #[test]
    fn w4_f3_the_unicorn_visual_rolls_one_type() {
        let mut app = rig(0x5EED);
        let b = dj_burst(ExtraJump::Unicorn);
        drain_double_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let mut rng = UnifiedRandom::new(0x5EED);
        let ty = [176, 177, 179][rng.next_max(3) as usize];
        let d = slots[0];
        assert_eq!(d.dust_type, ty);
        let _ = default_six(&mut rng, b.width as i32, b.height as i32, 0.0, 0.0, 1.0);
        assert_eq!(d.velocity, Vec2::ZERO);
        assert!(d.no_gravity);
        assert_eq!(d.scale, 0.5 + rng.next_float() * 0.8);
        assert_eq!(d.fade_in, 1.0 + rng.next_float() * 2.0);
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 9));
    }

    /// `:21948-21977` — the Sail visual: 2 puffs while `jump > 0`, 10 draws each
    /// (6 default + 1 scale + 3 dead vector); `velocity -= base.velocity/5`.
    #[test]
    fn w4_f3_the_sail_visual_keeps_the_dead_draws() {
        let mut app = rig(0x5EED);
        let b = dj_burst(ExtraJump::Sail);
        drain_double_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2, "jump > 0 → two puffs");
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 253);
            assert!(d.no_gravity);
            let (v, mut scale) = default_six(&mut rng, b.width as i32, 12, b.vx * 0.3, b.vy * 0.3, 1.5);
            scale += rng.next_range(-5, 3) as f32 * 0.1;
            assert_eq!(d.scale, scale);
            assert_eq!(d.velocity, v - Vec2::new(b.vx, b.vy) / 5.0);
            let _ = rng.next_range(-100, 101);
            let _ = rng.next_range(-100, 101);
            let _ = rng.next_max(81);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 2 * 10));
    }

    /// `:21978-22020` — the Blizzard visual: 2+3+3 type-76 puffs, noLight, the
    /// three velocity loops. 8×6 = 48 draws.
    #[test]
    fn w4_f3_the_blizzard_visual_runs_three_loops() {
        let mut app = rig(0x5EED);
        let b = dj_burst(ExtraJump::Blizzard);
        drain_double_jump_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 8);
        let v = Vec2::new(b.vx, b.vy);
        let mut rng = UnifiedRandom::new(0x5EED);
        for (i, d) in slots.iter().enumerate() {
            assert_eq!(d.dust_type, 76);
            assert!(d.no_gravity);
            assert!(d.no_light);
            let (vel, _) = default_six(&mut rng, b.width as i32, 12, b.vx * 0.3, b.vy * 0.3, 1.0);
            let expect = if i < 2 {
                let mut x = vel * 0.1;
                x += if i == 0 { v * 0.03 } else { -v * 0.03 };
                x -= v * 0.1;
                x
            } else if i < 5 {
                assert_eq!(d.fade_in, 1.5);
                vel * 0.6 + v * 0.8
            } else {
                assert_eq!(d.fade_in, 1.5);
                vel * 0.6 - v * 0.8
            };
            assert_eq!(d.velocity, expect);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 8 * 6));
    }

    /// `:25226-25233` — the petrify burst: 20 type-1 puffs, each + a `Next(2)`
    /// noGravity roll. 20×7 = 140 draws.
    #[test]
    fn w4_f3_the_stoned_burst_rolls_no_gravity_per_puff() {
        let mut app = rig(0x5EED);
        let b = StonedDust { x: 300.0, y: 600.0, width: 20.0, height: 42.0 };
        drain_stoned_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 20);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 1);
            let _ = default_six(&mut rng, b.width as i32, b.height as i32, 0.0, 0.0, 1.0);
            let roll = rng.next_max(2);
            assert_eq!(d.no_gravity, roll == 0);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 20 * 7));
    }

    /// `:23283-23316` — the drowning bubble: one type-34 puff, box `(width-8)×8`,
    /// scale 1.2. The type-34 wet-probe kill rides the pool (the rig's world is
    /// dry, so the slot deactivates but the 6 default draws still ride).
    #[test]
    fn w4_f3_the_drowning_bubble_spends_six_draws() {
        let mut app = rig(0x5EED);
        drain_drown_bubble(&mut app, Vec2::new(312.0, 604.0), 20.0);
        let mut rng = UnifiedRandom::new(0x5EED);
        let _ = default_six(&mut rng, 20 - 8, 8, 0.0, 0.0, 1.2);
        // The dry world's type-34 wet probe deactivates the slot (`Dust.cs:250`).
        assert!(live(&app).is_empty(), "the dry probe kills the type-34 bubble");
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 6));
    }

    // ---- W4-f3: MakeFloorDust witnesses -------------------------------------

    fn floor_burst(kind: i32, falling: bool, vx: f32) -> FloorDust {
        FloorDust {
            kind,
            frame_x: 0,
            frame_y: 0,
            paint: 0,
            falling,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx,
            vy: 0.0,
        }
    }

    /// `:23836` + `:23983-24037` — a plain snow floor (147 → type 76), walking:
    /// one puff, the `Next(100)` walking gate + the `Next(3)` scale arm.
    #[test]
    fn w4_f3_the_snow_floor_puffs_one_type_76() {
        let mut app = rig(0x5EED);
        let b = floor_burst(147, false, 3.0);
        drain_floor_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let mut rng = UnifiedRandom::new(0x5EED);
        // The walking gate: Next(100) > |vx|/3*100 = 100 → never refuses at vx 3.
        let _ = rng.next_max(100);
        let d = slots[0];
        assert_eq!(d.dust_type, 76);
        assert!(d.no_light);
        assert!(d.no_gravity);
        let (v, mut scale) = default_six(&mut rng, b.width as i32, 6, 0.0, 0.0, 1.0);
        scale += rng.next_max(3) as f32 * 0.1;
        assert_eq!(d.scale, scale);
        assert_eq!(d.velocity, v * 0.2);
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:23845-23847` + `:23921` — a sand floor (53 → type 32): the `Next(2)`
    /// gate can refuse the puff; the walking gate still draws.
    #[test]
    fn w4_f3_the_sand_floor_gates_on_next_two() {
        let mut app = rig(0x5EED);
        let b = floor_burst(53, false, 3.0);
        drain_floor_dust(&mut app, b);
        let mut rng = UnifiedRandom::new(0x5EED);
        let gate = rng.next_max(2);
        let _ = rng.next_max(100); // the walking gate draws regardless
        if gate == 0 {
            assert!(live(&app).is_empty(), "Next(2)==0 refuses the sand puff");
        } else {
            let slots = live(&app);
            assert_eq!(slots.len(), 1);
            assert_eq!(slots[0].dust_type, 32);
            let _ = default_six(&mut rng, b.width as i32, 6, 0.0, 0.0, 1.0);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:23825-23827` + `:24019-24031` — a falling landing on snow: 20 puffs,
    /// the falling velocity post-arms (no walking gate).
    #[test]
    fn w4_f3_the_falling_landing_bursts_twenty() {
        let mut app = rig(0x5EED);
        let b = floor_burst(147, true, 3.0);
        drain_floor_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 20);
        let mut rng = UnifiedRandom::new(0x5EED);
        let num5 = b.vx.clamp(-6.0, 6.0);
        for d in &slots {
            assert_eq!(d.dust_type, 76);
            let (v, mut scale) = default_six(&mut rng, b.width as i32, 6, 0.0, 0.0, 1.0);
            scale += rng.next_max(3) as f32 * 0.1; // the 76 arm
            // The falling post-arms (`:24019-24031`).
            let mut vel = v;
            vel.x *= 1.2;
            vel.y *= 0.8;
            vel.y -= 1.0;
            vel *= 0.8;
            scale += rng.next_max(3) as f32 * 0.1;
            vel.x = (d.position.x - (b.x + b.width * 0.5)) * 0.2;
            if vel.y > 0.0 {
                vel.y *= -1.0;
            }
            vel.x += num5 * 0.3;
            assert_eq!(d.scale, scale);
            assert_eq!(d.velocity, vel);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:23837-23844` — a painted shimmer-adjacent tile (666, paint != 0) breaks
    /// the loop before any draw.
    #[test]
    fn w4_f3_the_painted_666_breaks_without_draws() {
        let mut app = rig(0x5EED);
        let mut b = floor_burst(666, true, 3.0);
        b.paint = 5;
        drain_floor_dust(&mut app, b);
        assert!(live(&app).is_empty());
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 0));
    }

    // ---- W4-f3: wing-glide witnesses ----------------------------------------

    fn glide_burst(wings: i32) -> WingGlideDust {
        WingGlideDust {
            wings,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 3.0,
            vy: 2.0,
            facing: 1,
            always_animated: false,
        }
    }

    /// `:27040-27053` — wings 10: the `Next(3)` gate, then one type-76 puff with
    /// the fadeIn/noGravity/noLight/×0.3 post-arms.
    #[test]
    fn w4_f3_the_wings_10_glide_gates_on_next_three() {
        let mut app = rig(0x5EED);
        let b = glide_burst(10);
        drain_wing_glide(&mut app, b);
        let mut rng = UnifiedRandom::new(0x5EED);
        let gate = rng.next_max(3);
        if gate == 0 {
            let slots = live(&app);
            assert_eq!(slots.len(), 1);
            let d = slots[0];
            assert_eq!(d.dust_type, 76);
            assert!(d.no_gravity);
            assert!(d.no_light);
            assert_eq!(d.fade_in, 1.1);
            let (v, _) = default_six(&mut rng, 30, 30, 0.0, 0.0, 0.6);
            assert_eq!(d.velocity, v * 0.3);
        } else {
            assert!(live(&app).is_empty());
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:27149-27191` — wings 4: the `Next(2)` position pick draws always, one
    /// type-6 puff with `velocity.X *= 0.3`, `velocity.Y += 10`, noGravity.
    #[test]
    fn w4_f3_the_wings_4_rocket_puff_always_draws() {
        let mut app = rig(0x5EED);
        let b = glide_burst(4);
        drain_wing_glide(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let mut rng = UnifiedRandom::new(0x5EED);
        let pick = rng.next_max(2);
        let d = slots[0];
        assert_eq!(d.dust_type, 6);
        assert!(d.no_gravity);
        // The position pick: facing 1, pick==1 → x = center - 20, y + 6.
        let (px, py) = if pick == 1 {
            (b.x + b.width * 0.5 - 20.0, b.y + b.height - 18.0 + 6.0)
        } else {
            (b.x + b.width * 0.5 - 26.0, b.y + b.height - 18.0)
        };
        let ex = px + rng.next_max(8 - 4) as f32 + 4.0;
        let ey = py + rng.next_max(8 - 4) as f32 + 4.0;
        assert_eq!(d.position, Vec2::new(ex, ey));
        let vx = rng.next_range(-20, 21) as f32 * 0.1;
        let _vy = rng.next_range(-20, 21) as f32 * 0.1;
        let _ = rng.next_max(3);
        let _ = rng.next_range(-20, 21);
        // The type-6 tweak (`Dust.cs:214-227`): velocity.y = Next(-10,6)*0.1,
        // velocity.x *= 0.3, scale *= 0.7 — then the glide post-arms.
        let tweak_vy = rng.next_range(-10, 6) as f32 * 0.1;
        assert_eq!(d.velocity, Vec2::new(vx * 0.3 * 0.3, tweak_vy + 10.0));
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:27240-27273` — wings 26: one type-217 puff + the `Next(2)` second puff,
    /// `velocity /= 4` then `- base.velocity`.
    #[test]
    fn w4_f3_the_wings_26_glide_trails_217() {
        let mut app = rig(0x5EED);
        let b = glide_burst(26);
        drain_wing_glide(&mut app, b);
        let slots = live(&app);
        let mut rng = UnifiedRandom::new(0x5EED);
        let v = Vec2::new(b.vx, b.vy);
        // The first puff (always).
        assert!(!slots.is_empty());
        let d = slots[0];
        assert_eq!(d.dust_type, 217);
        assert!(d.no_gravity);
        assert!(d.no_light);
        let (vel, _) = default_six(&mut rng, 18, b.height as i32, 0.0, 0.0, 1.4);
        assert_eq!(d.velocity, vel / 4.0 - v);
        // The second puff gates on Next(2).
        let second = rng.next_max(2);
        if second == 0 {
            assert_eq!(slots.len(), 2);
            let d2 = slots[1];
            let (vel2, _) = default_six(&mut rng, 12, (b.height / 2.0) as i32, 0.0, 0.0, 1.4);
            assert_eq!(d2.velocity, vel2 / 4.0 - v);
        } else {
            assert_eq!(slots.len(), 1);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    // ---- W4-f3: liquid splash witnesses -------------------------------------

    fn splash_burst(arm: SplashArm) -> SplashDust {
        SplashDust {
            arm,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            water_style: 0,
        }
    }

    /// `:27783-27805` — the shimmer entry: 50 type-308 puffs, the `Next(6)`
    /// colour pick per puff, scale 0.8. The type-308 tweak (`velocity *= 0.5`,
    /// `Y += 1`) rides the pool before the post-arms.
    #[test]
    fn w4_f3_the_shimmer_entry_rolls_six_colours() {
        let mut app = rig(0x5EED);
        let b = splash_burst(SplashArm::ShimmerEntry);
        drain_splash_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 50);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 308);
            assert!(d.no_gravity);
            assert_eq!(d.scale, 0.8);
            let (v, _) = default_six(&mut rng, b.width as i32 + 12, 24, 0.0, 0.0, 1.0);
            // The type-308 tweak, then the post-arms.
            let mut vel = v * 0.5;
            vel.y += 1.0;
            vel.y -= 4.0;
            vel.x *= 2.5;
            assert_eq!(d.velocity, vel);
            let pick = rng.next_max(6);
            let expect = match pick {
                0 => Color::new(255, 255, 210, 255),
                1 => Color::new(190, 245, 255, 255),
                2 => Color::new(255, 150, 255, 255),
                _ => Color::new(190, 175, 255, 255),
            };
            assert_eq!(d.color, expect);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:27810-27818` — the honey entry: 20 type-152 puffs, alpha 100, scale 1.3.
    /// The type-152 tweak (`velocity *= 0.1`, `Y = -0.5`) rides the pool.
    #[test]
    fn w4_f3_the_honey_entry_puffs_152() {
        let mut app = rig(0x5EED);
        let b = splash_burst(SplashArm::HoneyEntry);
        drain_splash_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 20);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 152);
            assert_eq!(d.alpha, 100);
            assert!(d.no_gravity);
            assert_eq!(d.scale, 1.3);
            let (v, _) = default_six(&mut rng, b.width as i32 + 12, 24, 0.0, 0.0, 1.0);
            let mut vel = v * 0.1;
            vel.y = -0.5;
            vel.y -= 1.0;
            vel.x *= 2.5;
            assert_eq!(d.velocity, vel);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:27823-27831` — the water entry: 50 `dustWater()` puffs (style 0 → 33),
    /// alpha 100, scale 0.8. The type-33 tweak (`alpha = 170`, `velocity *= 0.5`,
    /// `Y += 1`) rides the pool before the post-arms re-set alpha 100.
    #[test]
    fn w4_f3_the_water_entry_uses_dust_water() {
        let mut app = rig(0x5EED);
        let b = splash_burst(SplashArm::WaterEntry);
        drain_splash_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 50);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 33, "water style 0 → dustWater 33");
            assert_eq!(d.alpha, 100);
            assert!(d.no_gravity);
            assert_eq!(d.scale, 0.8);
            let (v, _) = default_six(&mut rng, b.width as i32 + 12, 24, 0.0, 0.0, 1.0);
            let mut vel = v * 0.5;
            vel.y += 1.0;
            vel.y -= 3.0;
            vel.x *= 2.5;
            assert_eq!(d.velocity, vel);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:27837-27845` — the lava entry: 20 type-35 puffs, alpha 100, scale 1.3.
    #[test]
    fn w4_f3_the_lava_entry_puffs_35() {
        let mut app = rig(0x5EED);
        let b = splash_burst(SplashArm::LavaEntry);
        drain_splash_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 20);
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 35);
            assert_eq!(d.alpha, 100);
            assert_eq!(d.scale, 1.3);
            let (v, _) = default_six(&mut rng, b.width as i32 + 12, 24, 0.0, 0.0, 1.0);
            let mut vel = v * 0.1;
            vel.y = -0.5;
            vel.y -= 1.5;
            vel.x *= 2.5;
            assert_eq!(d.velocity, vel);
        }
        assert_eq!(stream_of(&mut app), rng);
    }

    // ---- W4-f3: wall-climb/wallslide dust witnesses -------------------------

    fn wall_burst(slide_dir: i32, rat_mount: bool) -> WallDust {
        WallDust {
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            slide_dir,
            rat_mount,
        }
    }

    /// `:21654`/`:21729`/`:21826` — the wall puff: one type-31, box 8×8 at the
    /// wall-side corner, `velocity *= 0.1`, `scale *= 1.2`, `noGravity`. The
    /// right wall (slideDir 1) keeps the X; 6 draws, no post-roll.
    #[test]
    fn w4_f3_the_wall_puff_mirrors_the_type_31_chain() {
        let mut app = rig(0x5EED);
        let b = wall_burst(1, false);
        drain_wall_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let d = slots[0];
        assert_eq!(d.dust_type, 31);
        assert!(d.no_gravity);
        let mut rng = UnifiedRandom::new(0x5EED);
        let (v, scale) = default_six(&mut rng, 8, 8, 0.0, 0.0, 1.0);
        assert_eq!(d.velocity, v * 0.1);
        assert_eq!(d.scale, scale * 1.2);
        // The right-wall corner: x + width/2 + (width/2 - 4)*1 = x + width - 4.
        assert!(d.position.x >= b.x + b.width - 4.0 - 4.0);
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 6));
    }

    /// `:21655-21658` — the left wall (slideDir -1) shifts the puff X by -10
    /// AFTER the NewDust; the Rat (`flag`) halves the scale (`:21669`).
    #[test]
    fn w4_f3_the_left_wall_shifts_x_and_the_rat_halves_scale() {
        let mut app = rig(0x5EED);
        let b = wall_burst(-1, true);
        drain_wall_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let d = slots[0];
        let mut rng = UnifiedRandom::new(0x5EED);
        // The default position draws land in the 8×8 box at the corner; the
        // -10 shift is on top. Recompute the corner the drain used (INTEGER
        // halves — the g66 keep-one form; even hitbox here, same value).
        let hw = b.width as i32 / 2;
        let corner_x = b.x + hw as f32 + (hw - 4) as f32 * -1.0;
        let px = rng.next_max(8.max(5) - 4);
        let _ = rng.next_max(8.max(5) - 4);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_max(3);
        let scale = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * 1.0;
        // The pool's default position is `corner + Next(w-4) + 4` (`Dust.cs:159`).
        assert_eq!(d.position.x, corner_x + px as f32 + 4.0 - 10.0);
        assert_eq!(d.scale, scale * 1.2 * 0.5, "the Rat halves the scale");
        assert_eq!(stream_of(&mut app), rng);
    }

    /// W4-f3 retire keep-one — the ODD-hitbox corner: the C# halves are
    /// INTEGER divisions (`(float)(base.width / 2)`, `:21654`); a 21×43 box
    /// corners at hw=10/hh=21, NOT the float 10.5/21.5 (the g66 248f0ff4d
    /// verdict — f3's float halves drifted 0.5 px on odd hitboxes).
    #[test]
    fn w4_f3_the_odd_hitbox_corner_uses_integer_halves() {
        let mut app = rig(0x5EED);
        let b = WallDust {
            x: 300.0,
            y: 600.0,
            width: 21.0,
            height: 43.0,
            slide_dir: 1,
            rat_mount: false,
        };
        drain_wall_dust(&mut app, b);
        let d = live(&app)[0];
        let mut rng = UnifiedRandom::new(0x5EED);
        // hw = 21/2 = 10, hh = 43/2 = 21: corner = (300+10+6, 600+21+17).
        let px = rng.next_max(8.max(5) - 4);
        let py = rng.next_max(8.max(5) - 4);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_range(-20, 21);
        let _ = rng.next_max(3);
        let _ = rng.next_range(-20, 21);
        assert_eq!(d.position.x, 316.0 + px as f32 + 4.0, "hw=10 not 10.5");
        assert_eq!(d.position.y, 638.0 + py as f32 + 4.0, "hh=21 not 21.5");
        assert_eq!(stream_of(&mut app), rng);
    }

    // ---- W4-f3: Vortex-stealth transition ring witnesses --------------------

    fn vortex_burst() -> VortexStealthDust {
        VortexStealthDust { cx: 310.0, cy: 621.0 }
    }

    /// `:25872-25893` — both gates pass: the type-229 outer ring (spin + 6
    /// default + Next(5,11) + NextFloat) then the type-240 inner ring (spin +
    /// 6 default + NextFloat). Draw order: gate, 2×NextDouble, 6, Next(5,11),
    /// NextFloat, gate, 2×NextDouble, 6, NextFloat.
    #[test]
    fn w4_f3_the_vortex_ring_replays_both_gates_in_order() {
        // Find a seed where both Next(2) gates pass (the first and the one
        // after the outer ring's draws).
        let seed = (0..200i32)
            .find(|&s| {
                let mut r = UnifiedRandom::new(s);
                if r.next_max(2) != 0 {
                    return false;
                }
                let _ = r.next_double();
                let _ = r.next_double();
                for _ in 0..6 {
                    r.next();
                }
                let _ = r.next_range(5, 11);
                let _ = r.next_float();
                r.next_max(2) == 0
            })
            .expect("a seed with both gates open");
        let mut app = rig(seed);
        let b = vortex_burst();
        drain_vortex_stealth(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].dust_type, 229);
        assert_eq!(slots[1].dust_type, 240);
        let center = Vec2::new(b.cx, b.cy);
        let tau = 6.2831854820251465f64;
        let half_pi = 1.5707963705062866f64;
        let mut rng = UnifiedRandom::new(seed);
        // Outer ring.
        assert_eq!(rng.next_max(2), 0);
        let a1 = rng.next_double() * tau - rng.next_double() * tau;
        let v1 = crate::bow_launch::rotated_by(Vec2::new(0.0, 1.0), a1);
        let _ = default_six(&mut rng, 0, 0, 0.0, 0.0, 1.0);
        let radius = rng.next_range(5, 11) as f32;
        let s1 = 0.5 + rng.next_float();
        assert!(slots[0].no_gravity);
        assert_eq!(slots[0].position, center - v1 * radius);
        assert_eq!(slots[0].velocity, crate::bow_launch::rotated_by(v1, half_pi) * 4.0);
        assert_eq!(slots[0].scale, s1);
        assert_eq!(slots[0].fade_in, 0.5);
        // Inner ring.
        assert_eq!(rng.next_max(2), 0);
        let a2 = rng.next_double() * tau - rng.next_double() * tau;
        let v2 = crate::bow_launch::rotated_by(Vec2::new(0.0, 1.0), a2);
        let _ = default_six(&mut rng, 0, 0, 0.0, 0.0, 1.0);
        let s2 = 0.5 + rng.next_float();
        assert!(slots[1].no_gravity);
        assert_eq!(slots[1].position, center - v2 * 12.0);
        assert_eq!(slots[1].velocity, crate::bow_launch::rotated_by(v2, -half_pi) * 2.0);
        assert_eq!(slots[1].scale, s2);
        assert_eq!(slots[1].fade_in, 0.5);
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:25874`/`:25884` — a refused gate spends NO draws beyond the gate
    /// itself. Both gates refuse → 2 draws, zero slots.
    #[test]
    fn w4_f3_the_vortex_gates_refuse_without_dust() {
        let seed = (0..200i32)
            .find(|&s| {
                let mut r = UnifiedRandom::new(s);
                r.next_max(2) != 0 && r.next_max(2) != 0
            })
            .expect("a seed with both gates closed");
        let mut app = rig(seed);
        drain_vortex_stealth(&mut app, vortex_burst());
        assert!(live(&app).is_empty());
        assert_eq!(stream_of(&mut app), draws_after(seed, 2));
    }
}
