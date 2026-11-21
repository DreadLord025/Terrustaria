//! W4-g66 — the player movement-trail dust drains: the per-tick
//! `DoubleJumpVisuals` arms (`Player.cs:21892-22021`) and the `MakeFloorDust`
//! landing/running chain (`:23790-24043`). Same queue convention as
//! `player_dust` (W4-d51): the tick snapshots WHERE each burst goes at the
//! C# read moment; this module replays each `Dust.NewDust` on the shared
//! `UpdatePlayers` stream (`Main.SwapRandom`, `Main.cs:17755-17757`) in exact
//! C# draw order. The pool is the App's `particles::DustPool`, so the
//! pressure bands / default block / ChildSafety / per-type tweaks ride
//! `DustPool::new_dust` verbatim.
//!
//! `gravDir` is the vanilla `1f` (no gravity flip in the port), so the
//! `gravDir == -1` folds collapse to their `+1` arms. The Sandstorm
//! `Gore.NewGore` sibling (`:21922-21924`) is the gore lane's (W4-d55) — its
//! `Main.rand.Next(220,223)` draw is a documented skip, as everywhere else
//! the port treats unported gore (docs/ANALYSIS.md).

use crate::app::App;
use crate::player::{DoubleJumpDust, ExtraJump, FloorDust};
use crate::player_dust::nd;
use glam::Vec2;
use tr_core::particles::Color;

/// Drain the `DoubleJumpVisuals` trail (`Player.cs:21892-22021`). The tick
/// already applied the per-arm gates (performing + option + !canJumpAgain +
/// the ascent sign) and the Sandstorm `sandStorm = true` re-arm (`:21919`);
/// this replays the surviving arm's dust chain. Called after the dash burst
/// drain (`:26767` follows `:26757`).
pub(crate) fn drain_double_jump_visuals(app: &mut App, b: DoubleJumpDust) {
    match b.kind {
        ExtraJump::Cloud => djv_cloud(app, &b),
        ExtraJump::Sandstorm => djv_sandstorm(app, &b),
        ExtraJump::Fart => djv_fart(app, &b),
        ExtraJump::Unicorn => djv_unicorn(app, &b),
        ExtraJump::Sail => djv_sail(app, &b),
        ExtraJump::Blizzard => djv_blizzard(app, &b),
        // `:21892-22021` has no Santank/Goat/Basilisk arm — their trails are
        // the one-shot start burst only (W4-d51's `drain_jump_dust`).
        ExtraJump::Santank | ExtraJump::WallOfFleshGoat | ExtraJump::Basilisk => {}
    }
}

/// `:21894-21904` — the Cloud's one type-16 puff at `(x-4, y+height)`, box
/// `width+8 × 4`, speed `(-vx*0.5, vy*0.5)`, alpha 100, scale 1.5. Post:
/// `velocity.X = vX*0.5 - vx*0.1`, `velocity.Y = vY*0.5 - vy*0.3`.
fn djv_cloud(app: &mut App, b: &DoubleJumpDust) {
    let idx = nd(
        app,
        Vec2::new(b.x - 4.0, b.y + b.height),
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

/// `:21905-21926` — the Sandstorm's three type-124 puffs at `(x, y+height/2)`,
/// box `width × 32`, speed `(vx*0.3, vy*0.3)`, alpha 150, scale
/// `(jump/75 + 1)/2`. Post per puff: `velocity *= 0.5*num4`,
/// `fadeIn = 1.5*num4`. The `miscCounter % 3` gore sibling (`:21920-21925`)
/// is W4-d55's — its `Next(220,223)` draw is a documented skip.
fn djv_sandstorm(app: &mut App, b: &DoubleJumpDust) {
    let num4 = (b.jump as f32 / 75.0 + 1.0) / 2.0;
    for _ in 0..3 {
        let idx = nd(
            app,
            Vec2::new(b.x, b.y + (b.height as i32 / 2) as f32),
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
}

/// `:21927-21938` — the Fart's one type-188 puff, the Cloud shape plus the
/// trailing `velocity *= 0.5`.
fn djv_fart(app: &mut App, b: &DoubleJumpDust) {
    let idx = nd(
        app,
        Vec2::new(b.x - 4.0, b.y + b.height),
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

/// `:21939-21947` — the Unicorn's one `SelectRandom(176,177,179)` puff over
/// the whole body box. Post: velocity zeroed, `noGravity`,
/// `scale = 0.5 + NextFloat*0.8`, `fadeIn = 1 + NextFloat*2`.
fn djv_unicorn(app: &mut App, b: &DoubleJumpDust) {
    let ty = {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        [176, 177, 179][rng.next_max(3) as usize]
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

/// `:21948-21977` — the Sail's 1-or-2 type-253 puffs (`jump > 0` → 2) at
/// `(x, y+height-6)`, box `width × 12`, speed `(vx*0.3, vy*0.3)`. Post per
/// puff: `scale += Next(-5,3)*0.1`, then `jump <= 0` → `scale *= 0.8` else
/// `velocity -= playerVel/5`; `noGravity`; then the DEAD `vector` draws
/// (`Next(-100,101)`×2 + `Next(81)` — normalised/scaled, never read).
fn djv_sail(app: &mut App, b: &DoubleJumpDust) {
    let count = if b.jump > 0 { 2 } else { 1 };
    for _ in 0..count {
        let idx = nd(
            app,
            Vec2::new(b.x, b.y + b.height - 6.0),
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

/// `:21978-22020` — the Blizzard's 2+3+3 type-76 puffs at `(x, y+height-6)`,
/// box `width × 12`, speed `(vx*0.3, vy*0.3)`, default alpha/scale. The
/// first pair: `velocity *= 0.1`, ±`playerVel*0.03` (k=0 add, k=1 sub), then
/// `-= playerVel*0.1`; the two triples: `fadeIn = 1.5`, `velocity *= 0.6`,
/// ±`playerVel*0.8`. All: `noGravity`, `noLight`.
fn djv_blizzard(app: &mut App, b: &DoubleJumpDust) {
    let pos = Vec2::new(b.x, b.y + b.height - 6.0);
    let vel = Vec2::new(b.vx, b.vy);
    for k in 0..2 {
        let idx = nd(
            app,
            pos,
            b.width as i32,
            12,
            76,
            b.vx * 0.3,
            b.vy * 0.3,
            0,
            Color::default(),
            1.0,
        );
        let d = app.dust.slot_mut(idx);
        d.velocity *= 0.1;
        if k == 0 {
            d.velocity += vel * 0.03;
        } else {
            d.velocity -= vel * 0.03;
        }
        d.velocity -= vel * 0.1;
        d.no_gravity = true;
        d.no_light = true;
    }
    for sign in [1.0f32, -1.0] {
        for _ in 0..3 {
            let idx = nd(
                app,
                pos,
                b.width as i32,
                12,
                76,
                b.vx * 0.3,
                b.vy * 0.3,
                0,
                Color::default(),
                1.0,
            );
            let d = app.dust.slot_mut(idx);
            d.fade_in = 1.5;
            d.velocity *= 0.6;
            d.velocity += vel * 0.8 * sign;
            d.no_gravity = true;
            d.no_light = true;
        }
    }
}

/// Drain the `MakeFloorDust` chain (`Player.cs:23790-24043`). The tick
/// snapshotted the floor tile (post slope carve-out) and the pre-integration
/// `falling` flag; the shimmer-block ParticleOrchestrator arm
/// (`:23792-23818`, types 659/667) is the particle-orchestra lane's — its
/// `Next(100)`/two `NextFloat`/`NextVector2Circular` draws are a documented
/// skip here (no orchestrator in the port yet). The landing Item177 cue
/// (`:23830`) rides the drain BEFORE the puff loop, on the same stream.
pub(crate) fn drain_floor_dust(app: &mut App, b: FloorDust) {
    let ty = b.kind;
    // `:23819-23823`: the type gate — the framed-block row 49 (`type 19,
    // frameY/18 == 49`) joins the plain set.
    let framed_row49 = ty == 19 && b.frame_y / 18 == 49;
    let allowed = matches!(
        ty,
        147 | 25
            | 53
            | 189
            | 0
            | 123
            | 57
            | 112
            | 116
            | 196
            | 193
            | 195
            | 197
            | 199
            | 229
            | 234
            | 371
            | 460
            | 666
            | 712
            | 717
            | 718
            | 719
    ) || framed_row49;
    if !allowed {
        return;
    }
    let count = if b.falling { 20 } else { 1 };
    // `:23828-23831`: the Stinky-tile landing cue — Item177 at
    // `(Center.X, Bottom.Y)`, before the loop's draws.
    if b.falling && (ty == 666 || ty == 712) {
        if let Some(idx) = crate::soundid::SOUND_STYLES
            .iter()
            .position(|s| s.name == "Item177")
        {
            app.audio.play_use_sound(
                idx as u16,
                b.x + b.width * 0.5,
                b.y + b.height,
                0.0,
                app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            );
        }
    }
    // `:23837-23844`: the painted Stinky tiles `break` the WHOLE loop before
    // any puff draws (the cue above already played).
    if (ty == 666 || ty == 712) && b.paint != 0 {
        return;
    }
    for _ in 0..count {
        floor_puff(app, &b, framed_row49, count);
    }
}

/// One `MakeFloorDust` iteration (`:23833-24042`): the dust-type remap, the
/// per-type skip rolls, the run-speed gate, the spawn and the post-arms.
fn floor_puff(app: &mut App, b: &FloorDust, framed_row49: bool, count: i32) {
    let ty = b.kind;
    // `:23836-23920`: the tile → dust-type remap, in source order (later
    // ifs overwrite earlier ones; the sets are disjoint in practice).
    let mut num3 = 76;
    if ty == 666 || ty == 712 {
        num3 = 322;
    }
    if ty == 53 {
        num3 = 32;
    }
    if ty == 189 || framed_row49 {
        num3 = 16;
    }
    if ty == 717 {
        num3 = 326;
    }
    if ty == 718 {
        num3 = 327;
    }
    if ty == 719 {
        num3 = 328;
    }
    if ty == 0 {
        num3 = 0;
    }
    if ty == 123 {
        num3 = 53;
    }
    if ty == 57 {
        num3 = 36;
    }
    if ty == 112 {
        num3 = 14;
    }
    if ty == 234 {
        num3 = 122;
    }
    if ty == 116 {
        num3 = 51;
    }
    if ty == 196 {
        num3 = 108;
    }
    if ty == 193 {
        num3 = 4;
    }
    if ty == 195 || ty == 199 {
        num3 = 5;
    }
    if ty == 197 {
        num3 = 4;
    }
    if ty == 229 {
        num3 = 153;
    }
    if ty == 371 {
        num3 = 243;
    }
    if ty == 460 {
        num3 = 108;
    }
    if ty == 25 {
        num3 = 37;
    }
    // `:23921-23944`: the per-type skip rolls (each draws even when the
    // earlier ones already refused — the C# keeps rolling).
    let mut keep = true;
    {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        if num3 == 32 && rng.next_max(2) == 0 {
            keep = false;
        }
        if num3 == 14 && rng.next_max(2) == 0 {
            keep = false;
        }
        if num3 == 51 && rng.next_max(2) == 0 {
            keep = false;
        }
        if num3 == 36 && rng.next_max(2) == 0 {
            keep = false;
        }
        if num3 == 0 && rng.next_max(3) != 0 {
            keep = false;
        }
        if num3 == 53 && rng.next_max(3) != 0 {
            keep = false;
        }
        // `:23958-23965`: the run-speed gate — `Next(100) > |vx|/3*100`.
        if !b.falling {
            let num4 = b.vx.abs() / 3.0;
            if rng.next_max(100) as f32 > num4 * 100.0 {
                keep = false;
            }
        }
    }
    if !keep {
        return;
    }
    // `:23945-23957`: the per-type colour.
    let color = match ty {
        193 => Color::new(30, 100, 255, 100),
        197 => Color::new(97, 200, 255, 100),
        460 => Color::new(100, 150, 130, 100),
        _ => Color::default(),
    };
    // `:23970-23982`: the clamped run speed and the walking gate.
    let num5 = b.vx.clamp(-6.0, 6.0);
    let walking = b.vx.abs() + b.vy.abs() > 1.0;
    if !(walking | b.falling) {
        return;
    }
    let idx = nd(
        app,
        Vec2::new(b.x, b.y + b.height - 2.0),
        b.width as i32,
        6,
        num3,
        0.0,
        0.0,
        50,
        color,
        1.0,
    );
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
    let d = app.dust.slot_mut(idx);
    // `:23988-24005`: the per-type scale/alpha post-arms.
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
    // `:24019-24036`: the landing burst's spread vs the running puff.
    if count > 1 {
        d.velocity.x *= 1.2;
        d.velocity.y *= 0.8;
        d.velocity.y -= 1.0;
        d.velocity *= 0.8;
        d.scale += rng.next_max(3) as f32 * 0.1;
        d.velocity.x = (d.position.x - (b.x + b.width * 0.5)) * 0.2;
        if d.velocity.y > 0.0 {
            d.velocity.y *= -1.0;
        }
        d.velocity.x += num5 * 0.3;
    } else {
        d.velocity *= 0.2;
    }
    d.position.x -= num5;
}#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use tr_core::{UnifiedRandom, World};

    /// The W4-d51 drain rig: empty world, full 6000 budget (no pressure-band
    /// draws below slot 3000), camera parked on the burst, `UpdatePlayers`
    /// pinned to `seed`.
    fn rig(seed: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("dust".into(), 64, 128);
        app.gfx_quality.max_dust_to_draw = 6000;
        app.camera = Vec2::new(320.0, 640.0);
        *app.rngs.stream(crate::app::RngKey::UpdatePlayers) = UnifiedRandom::new(seed);
        app
    }

    fn burst(kind: ExtraJump) -> DoubleJumpDust {
        DoubleJumpDust {
            kind,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 3.0,
            vy: -5.01,
            jump: 11,
        }
    }

    fn live(app: &App) -> Vec<&tr_core::particles::Dust> {
        app.dust.slots().iter().filter(|d| d.active).collect()
    }

    fn stream_of(app: &mut App) -> UnifiedRandom {
        app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone()
    }

    /// The stream after `n` raw draws from `seed` (the draw-count contract).
    fn draws_after(seed: i32, n: usize) -> UnifiedRandom {
        let mut rng = UnifiedRandom::new(seed);
        for _ in 0..n {
            rng.next();
        }
        rng
    }

    /// One `NewDust` default block on the mirror stream (`Dust.cs:153-193`):
    /// position x/y, velocity x/y, frame row, scale — six draws.
    fn default_block(
        rng: &mut UnifiedRandom,
        pos: Vec2,
        w: i32,
        h: i32,
        sx: f32,
        sy: f32,
        scale: f32,
    ) -> (Vec2, Vec2, f32) {
        let w = w.max(5);
        let h = h.max(5);
        let px = pos.x + rng.next_max(w - 4) as f32 + 4.0;
        let py = pos.y + rng.next_max(h - 4) as f32 + 4.0;
        let vx = rng.next_range(-20, 21) as f32 * 0.1 + sx;
        let vy = rng.next_range(-20, 21) as f32 * 0.1 + sy;
        let _ = rng.next_max(3);
        let sc = (1.0 + rng.next_range(-20, 21) as f32 * 0.01) * scale;
        (Vec2::new(px, py), Vec2::new(vx, vy), sc)
    }

    /// `:21894-21904` — the Cloud trail: one type-16 puff, six draws, the
    /// velocity folds (no post draws).
    #[test]
    fn w4_g66_the_cloud_trail_folds_the_velocity() {
        let mut app = rig(0x5EED);
        let b = burst(ExtraJump::Cloud);
        drain_double_jump_visuals(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let d = slots[0];
        assert_eq!(d.dust_type, 16);
        let mut rng = UnifiedRandom::new(0x5EED);
        let (pos, vel, sc) = default_block(
            &mut rng,
            Vec2::new(b.x - 4.0, b.y + b.height),
            28,
            4,
            -b.vx * 0.5,
            b.vy * 0.5,
            1.5,
        );
        assert_eq!(d.position, pos);
        assert_eq!(d.scale, sc);
        assert_eq!(
            d.velocity,
            Vec2::new(vel.x * 0.5 - b.vx * 0.1, vel.y * 0.5 - b.vy * 0.3)
        );
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 6));
    }

    /// `:21905-21918` — the Sandstorm trail: three type-124 puffs, scale
    /// `(jump/75+1)/2`, the velocity/fadeIn folds; 18 draws total.
    #[test]
    fn w4_g66_the_sandstorm_trail_scales_by_the_hold_counter() {
        let mut app = rig(0x5EED);
        let b = burst(ExtraJump::Sandstorm);
        drain_double_jump_visuals(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 3);
        let num4 = (b.jump as f32 / 75.0 + 1.0) / 2.0;
        let mut rng = UnifiedRandom::new(0x5EED);
        for d in &slots {
            assert_eq!(d.dust_type, 124);
            let (pos, vel, sc) = default_block(
                &mut rng,
                Vec2::new(b.x, b.y + 21.0),
                20,
                32,
                b.vx * 0.3,
                b.vy * 0.3,
                num4,
            );
            assert_eq!(d.position, pos);
            assert_eq!(d.velocity, vel * 0.5 * num4);
            assert_eq!(d.scale, sc);
            assert_eq!(d.fade_in, 1.5 * num4);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 18));
    }

    /// `:21939-21947` — the Unicorn trail: the SelectRandom type roll, the
    /// zeroed velocity, the scale/fadeIn floats (9 draws).
    #[test]
    fn w4_g66_the_unicorn_trail_rolls_type_scale_and_fade() {
        let mut app = rig(0x5EED);
        let b = burst(ExtraJump::Unicorn);
        drain_double_jump_visuals(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let d = slots[0];
        let mut rng = UnifiedRandom::new(0x5EED);
        let ty = [176, 177, 179][rng.next_max(3) as usize];
        assert_eq!(d.dust_type, ty);
        let (pos, _, _) =
            default_block(&mut rng, Vec2::new(b.x, b.y), 20, 42, 0.0, 0.0, 1.0);
        assert_eq!(d.position, pos);
        assert_eq!(d.velocity, Vec2::ZERO);
        assert!(d.no_gravity);
        assert_eq!(d.scale, 0.5 + rng.next_float() * 0.8);
        assert_eq!(d.fade_in, 1.0 + rng.next_float() * 2.0);
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 9));
    }

    /// `:21948-21977` — the Sail trail: `jump > 0` → two puffs with the
    /// velocity fold; `jump <= 0` → one with the ×0.8 scale. Each puff:
    /// six default + one scale roll + three dead vector draws.
    #[test]
    fn w4_g66_the_sail_trail_splits_on_the_hold_counter() {
        for jump in [11, 0] {
            let mut app = rig(0x5EED);
            let mut b = burst(ExtraJump::Sail);
            b.jump = jump;
            drain_double_jump_visuals(&mut app, b);
            let slots = live(&app);
            let count = if jump > 0 { 2 } else { 1 };
            assert_eq!(slots.len(), count);
            let mut rng = UnifiedRandom::new(0x5EED);
            for d in &slots {
                assert_eq!(d.dust_type, 253);
                let (pos, vel, mut sc) = default_block(
                    &mut rng,
                    Vec2::new(b.x, b.y + b.height - 6.0),
                    20,
                    12,
                    b.vx * 0.3,
                    b.vy * 0.3,
                    1.5,
                );
                sc += rng.next_range(-5, 3) as f32 * 0.1;
                let mut v = vel;
                if jump <= 0 {
                    sc *= 0.8;
                } else {
                    v -= Vec2::new(b.vx, b.vy) / 5.0;
                }
                let _ = rng.next_range(-100, 101);
                let _ = rng.next_range(-100, 101);
                let _ = rng.next_max(81);
                assert_eq!(d.position, pos);
                assert_eq!(d.velocity, v);
                assert_eq!(d.scale, sc);
                assert!(d.no_gravity);
            }
            assert_eq!(stream_of(&mut app), rng);
            assert_eq!(stream_of(&mut app), draws_after(0x5EED, count * 10));
        }
    }

    /// `:21978-22020` — the Blizzard trail: 2+3+3 type-76 puffs, the ±0.03 /
    /// ±0.8 velocity splits, fadeIn 1.5 on the triples; 48 draws.
    #[test]
    fn w4_g66_the_blizzard_trail_splits_eight_puffs() {
        let mut app = rig(0x5EED);
        let b = burst(ExtraJump::Blizzard);
        drain_double_jump_visuals(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 8);
        let vel = Vec2::new(b.vx, b.vy);
        let mut rng = UnifiedRandom::new(0x5EED);
        for (i, d) in slots.iter().enumerate() {
            assert_eq!(d.dust_type, 76);
            let (pos, v, sc) = default_block(
                &mut rng,
                Vec2::new(b.x, b.y + b.height - 6.0),
                20,
                12,
                b.vx * 0.3,
                b.vy * 0.3,
                1.0,
            );
            assert_eq!(d.position, pos);
            assert_eq!(d.scale, sc);
            let expect = if i < 2 {
                let mut v = v * 0.1;
                v += vel * 0.03 * if i == 0 { 1.0 } else { -1.0 };
                v -= vel * 0.1;
                assert_eq!(d.fade_in, 0.0);
                v
            } else {
                assert_eq!(d.fade_in, 1.5);
                v * 0.6 + vel * 0.8 * if i < 5 { 1.0 } else { -1.0 }
            };
            assert_eq!(d.velocity, expect);
            assert!(d.no_gravity && d.no_light);
        }
        assert_eq!(stream_of(&mut app), rng);
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 48));
    }

    /// `:23833-24042` — the sand landing burst: 20 iterations, each rolls
    /// `Next(2)` (type 32); the kept puffs spend six default + `Next(6)` +
    /// `Next(3)` draws and the spread post-arms. The mirror replays the
    /// exact gate order.
    #[test]
    fn w4_g66_the_sand_landing_burst_mirrors_the_gate_order() {
        let mut app = rig(0x5EED);
        let b = FloorDust {
            falling: true,
            kind: 53,
            frame_y: 0,
            paint: 0,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 9.0, // clamps to 6
            vy: 0.0,
        };
        drain_floor_dust(&mut app, b);
        let mut rng = UnifiedRandom::new(0x5EED);
        let mut kept = 0;
        let mut slot = 0;
        for _ in 0..20 {
            if rng.next_max(2) == 0 {
                continue;
            }
            kept += 1;
            let d = app.dust.slot(slot);
            slot += 1;
            assert!(d.active);
            assert_eq!(d.dust_type, 32);
            let (pos, vel, mut sc) = default_block(
                &mut rng,
                Vec2::new(b.x, b.y + b.height - 2.0),
                20,
                6,
                0.0,
                0.0,
                1.0,
            );
            assert_eq!(d.position, Vec2::new(pos.x - 6.0, pos.y));
            sc += rng.next_max(6) as f32 * 0.1;
            assert!(d.no_gravity);
            let mut v = vel;
            v.x *= 1.2;
            v.y *= 0.8;
            v.y -= 1.0;
            v *= 0.8;
            sc += rng.next_max(3) as f32 * 0.1;
            v.x = (pos.x - (b.x + 10.0)) * 0.2;
            if v.y > 0.0 {
                v.y *= -1.0;
            }
            v.x += 6.0 * 0.3;
            assert_eq!(d.velocity, v);
            assert_eq!(d.scale, sc);
        }
        assert_eq!(live(&app).len(), kept);
        assert_eq!(stream_of(&mut app), rng);
    }

    /// `:23958-23965` + `:23979-23982` — the running puff: the `Next(100)`
    /// speed gate and the walking gate refuse a standing player; a running
    /// one keeps the ×0.2 velocity arm.
    #[test]
    fn w4_g66_the_running_floor_puff_gates_on_speed() {
        // Standing still: the Next(100) gate refuses every roll > 0 and the
        // walking gate refuses the rest — nothing spawns either way.
        let mut app = rig(0x5EED);
        let mut b = FloorDust {
            falling: false,
            kind: 147, // snow -> type 76, no skip roll
            frame_y: 0,
            paint: 0,
            x: 300.0,
            y: 600.0,
            width: 20.0,
            height: 42.0,
            vx: 0.0,
            vy: 0.0,
        };
        drain_floor_dust(&mut app, b);
        assert_eq!(live(&app).len(), 0, "standing still spawns nothing");
        // The gate drew Next(100) once.
        assert_eq!(stream_of(&mut app), draws_after(0x5EED, 1));
        // Running: |vx| = 3 → num4*100 = 100, every Next(100) roll passes;
        // the walking gate passes too.
        let mut app = rig(0x5EED);
        b.vx = 3.0;
        drain_floor_dust(&mut app, b);
        let slots = live(&app);
        assert_eq!(slots.len(), 1);
        let d = slots[0];
        assert_eq!(d.dust_type, 76);
        let mut rng = UnifiedRandom::new(0x5EED);
        let _ = rng.next_max(100);
        let (pos, vel, mut sc) = default_block(
            &mut rng,
            Vec2::new(b.x, b.y + b.height - 2.0),
            20,
            6,
            0.0,
            0.0,
            1.0,
        );
        sc += rng.next_max(3) as f32 * 0.1;
        assert!(d.no_light && d.no_gravity);
        assert_eq!(d.scale, sc);
        assert_eq!(d.velocity, vel * 0.2);
        assert_eq!(d.position, Vec2::new(pos.x - 3.0, pos.y));
        assert_eq!(stream_of(&mut app), rng);
    }

    /// The tick side: a performing Cloud jump queues one trail snapshot per
    /// airborne tick while the ascent lasts; the `jump == 0` cancel
    /// (`:26753-26756`) stops it, and the option/can gates hold.
    #[test]
    fn w4_g66_the_tick_queues_the_performing_jump_trail() {
        let w = World::new("t".into(), 64, 128);
        let mut p = Player::spawn(&w);
        p.pinned_jumps[ExtraJump::Cloud.index()] = true;
        p.update(&w, crate::player::PlayerInput::default());
        p.can_jump_again[ExtraJump::Cloud.index()] = true;
        // The trigger: the start burst (W4-d51) + the first trail tick.
        p.update(
            &w,
            crate::player::PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert!(p.take_jump_dust().is_some(), "the start burst fired");
        let t = p
            .take_double_jump_dust()
            .expect("the trail queues on the ascent");
        assert_eq!(t.kind, ExtraJump::Cloud);
        assert!(t.vy < 0.0);
        assert!(t.jump > 0);
        // Held ticks keep queueing while the hold counter lasts.
        p.update(
            &w,
            crate::player::PlayerInput {
                jump: true,
                ..Default::default()
            },
        );
        assert!(p.take_double_jump_dust().is_some());
        // Spend the counter: the cancel clears the performing flag.
        p.jump = 0;
        p.update(&w, crate::player::PlayerInput::default());
        assert!(
            p.take_double_jump_dust().is_none(),
            "the cancel stops the trail"
        );
        assert_eq!(p.performing_jump, None);
    }

    /// The tick side: `FloorVisuals` queues the floor snapshot with the
    /// slope carve-out; a sloped row kills the dust kind but keeps the flags.
    #[test]
    fn w4_g66_the_floor_snapshot_carries_the_slope_carve_out() {
        use tr_core::tile::{Tile, FLAG_ACTIVE};
        let mut w = World::new("t".into(), 64, 128);
        // A sand row at y=60 under the spawn column.
        for x in 0..64 {
            let i = w.index(x, 60);
            w.tiles[i] = Tile {
                kind: 53,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.spawn_x = 10;
        w.spawn_y = 40;
        let mut p = Player::spawn(&w);
        for _ in 0..120 {
            p.update(&w, crate::player::PlayerInput::default());
        }
        let f = p.take_floor_dust().expect("grounded on sand queues the dust");
        assert_eq!(f.kind, 53);
        assert!(!f.falling, "settled ticks are not falling");
        // Slope the center tile: the flags stay, the dust kind dies.
        let i = w.index(((p.x + p.width * 0.5) / 16.0) as i32, 60);
        w.tiles[i].set_slope(1);
        p.update(&w, crate::player::PlayerInput::default());
        let f = p.take_floor_dust().expect("the snapshot still queues");
        assert_eq!(f.kind, -1, "the slope carve-out");
    }
}