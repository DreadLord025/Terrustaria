//! W4-d80 — the DD2 aiStyle-1 one-shot riders whose `AI_001` per-type hooks
//! carry a tracked sound: the Phantom Phoenix shot (706,
//! `Projectile.cs:55767-55863`) and the Sky Dragon's Fury shot (709,
//! `:54696-54725`). Both rest their flight counter (`flag3 = false`,
//! `:56024-56025`), so the generic `ai[0] >= 15` gravity arm never fires —
//! they fly straight at their spawn velocity.
//!
//! The tracked plays ride the `ProjTrackedCue` queue (the `wants_sound`
//! convention): the 706 stores its `DD2_PhantomPhoenixShot` handle into
//! `localAI[0]` and follows it every tick (`HandlePlayFollow`, idx 0); the
//! 709's `DD2_SkyDragonsFuryShot` is a fire-and-forget one-shot (`OneShot`)
//! on the `ai[1] == 0` tick. ZERO gameplay draws — the variation/pitch picks
//! ride the two audio-only statics (the census' RNG contract, research/64).
//!
//! Deliberate skips (the slice-B/C precedent, ANALYSIS): the dust bursts
//! (706's `:55778-55788`/`:55801-55835`/`:55849-55858`, 709 has none), the
//! `Lighting.AddLight`/`PlotTileLine` (706 `:55861-55862`, 709 `:54714`) —
//! no dust/light layers in the pool. The 706's wet kill (`base.wet`,
//! `:55769`) reads the centre-cell liquid stand-in (the `flail_in_liquid`
//! idiom — `WetCollision`'s box scan is the documented approximation).
//!
//! CODE-ONLY MODE (W4-d80): the witness tests at the bottom are WRITTEN NOT
//! RUN (no compiler invoked); verified by re-reading the cited C# blocks.

use tr_core::World;

use crate::projectile::{ProjTrackedCue, Projectile, PHANTOM_PHOENIX, SKY_DRAGONS_FURY_SHOT};

/// The slice's kind set — the two DD2 aiStyle-1 tracked-shot riders.
pub(crate) const DD2_SHOT_KINDS: [u16; 2] = [PHANTOM_PHOENIX, SKY_DRAGONS_FURY_SHOT];

/// Whether this module owns the kind's aiStyle-1 arm.
pub(crate) fn covers(kind: u16) -> bool {
    DD2_SHOT_KINDS.contains(&kind)
}

/// `AI_001`'s per-type body for the two DD2 tracked shots, in the C#'s own
/// block order. The flight counter rests (`flag3 = false`), the generic
/// gravity arm is unreachable, and the movement is the port's kill-on-tile
/// stand-in (`HandleMovement`'s default arm, the slice-B/C `move_only`).
pub(crate) fn step(p: &mut Projectile, world: &World) {
    match p.kind {
        PHANTOM_PHOENIX => phoenix(p, world),
        SKY_DRAGONS_FURY_SHOT => sky_dragons_fury(p),
        _ => unreachable!("the dispatch gates on DD2_SHOT_KINDS"),
    }
    // The shared tail: the +16 Y clamp (`flag8`, `:57753-57768`; `flag4` is
    // false — no slice kind sets it) then the kill-on-tile movement.
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    move_only(p, world);
}

/// The Phantom Phoenix shot (706, `Projectile.cs:55767-55863`): the wet
/// kill, the `localAI[1] == 0` one-shot play into `localAI[0]` + the
/// per-tick follow, the 40-tick frame machine (`frame = frameCounter / 5`),
/// the alpha fade 55 and the rotation tail (`:57597-57608`).
fn phoenix(p: &mut Projectile, world: &World) {
    // `:55769-55773` — the wet kill (`base.wet` = WetCollision; the port's
    // centre-cell liquid stand-in, the `flail_in_liquid` idiom).
    let cx = (p.pos.x + p.w * 0.5) / 16.0;
    let cy = (p.pos.y + p.h * 0.5) / 16.0;
    let wet = world
        .tile(cx as i32, cy as i32)
        .is_some_and(|t| t.liquid > 0);
    if wet {
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:55774-55798` — the one-shot play (`DD2_PhantomPhoenixShot`) into
    // `localAI[0]` on the `localAI[1] == 0` tick, then the per-tick follow
    // (null → Invalid, else Position = Center). The 15-dust burst
    // (`:55778-55788`) is skipped. W4-d80.
    let play = p.local_ai[1] == 0.0;
    if play {
        p.local_ai[1] += 1.0;
    }
    p.wants_tracked = Some(ProjTrackedCue::HandlePlayFollow {
        style: "DD2_PhantomPhoenixShot",
        idx: 0,
        play,
    });
    // `:55799-55836` — the alpha-0 dust window (skipped).
    // `:55837-55841` — the 40-tick frame machine: `frameCounter++`, wrap at
    // 40, `frame = frameCounter / 5` (the 8-row sheet, `Main.cs:8456`).
    p.frame_counter += 1;
    if p.frame_counter >= 40 {
        p.frame_counter = 0;
    }
    p.frame = p.frame_counter / 5;
    // `:55842-55860` — the alpha fade 55 (clamped at 0); the alpha-0 16-dust
    // ring (`:55849-55858`) is skipped.
    if p.alpha > 0 {
        p.alpha -= 55;
        if p.alpha < 0 {
            p.alpha = 0;
        }
    }
    // `:55861-55862` — the light cast (skipped).
    // `:57597-57608` — the rotation tail: `spriteDirection = direction`,
    // the velocity angle (the direction < 0 flip). The velocity-sign
    // stand-in for `base.direction` (the slice-B convention).
    let dir = if p.vel.x < 0.0 { -1 } else { 1 };
    p.sprite_direction = dir;
    p.rot = if dir < 0 {
        (0.0 - p.vel.y).atan2(0.0 - p.vel.x)
    } else {
        p.vel.y.atan2(p.vel.x)
    };
}

/// The Sky Dragon's Fury shot (709, `Projectile.cs:54696-54725`): the alpha
/// fade 30, the 3-tick frame machine (wrap 3), the `ai[1] == 0` one-shot
/// (`DD2_SkyDragonsFuryShot`), the `ai[1]++` age and the >= 30 kill, then
/// the rotation tail (`:57642-57649`).
fn sky_dragons_fury(p: &mut Projectile) {
    // `:54698-54702` — the alpha fade 30, clamped at 0.
    p.alpha -= 30;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:54703` — `spriteDirection = direction` (the velocity-sign stand-in).
    p.sprite_direction = if p.vel.x < 0.0 { -1 } else { 1 };
    // `:54704-54713` — the 3-tick frame machine: `frameCounter++`, at >= 3
    // `frame++` + reset, wrap `frame >= 3` to 0 (the 3-row sheet,
    // `Main.cs:8554`).
    p.frame_counter += 1;
    if p.frame_counter >= 3 {
        p.frame += 1;
        p.frame_counter = 0;
        if p.frame >= 3 {
            p.frame = 0;
        }
    }
    // `:54714` — the light cast (skipped).
    // `:54715-54718` — the one-shot (`DD2_SkyDragonsFuryShot`) on the
    // `ai[1] == 0` tick, the handle discarded. W4-d80.
    if p.ai[1] == 0.0 {
        p.wants_tracked = Some(ProjTrackedCue::OneShot("DD2_SkyDragonsFuryShot"));
    }
    // `:54719-54724` — the age counter + the >= 30 kill.
    p.ai[1] += 1.0;
    if p.ai[1] >= 30.0 {
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:57642-57649` — the rotation tail: the velocity angle, + π when
    // direction == -1 (the velocity-sign stand-in).
    p.rot = p.vel.y.atan2(p.vel.x);
    if p.vel.x < 0.0 {
        p.rot += std::f32::consts::PI;
    }
}

/// The movement stand-in — a verbatim replication of `ai_style1_flight`'s
/// integration (the `HandleMovement` default arm `Projectile.cs:19843-
/// 19848`, the slice-B/C `move_only`). Both kinds keep `tileCollide = true`.
fn move_only(p: &mut Projectile, world: &World) {
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            p.active = false;
            p.time_left = 0;
        } else {
            p.pos.x = nx;
            p.pos.y = ny;
        }
    } else {
        p.pos += p.vel;
    }
}

// ---------------------------------------------------------------------------
// Witness tests — WRITTEN NOT RUN (code-only mode, W4-d80). Verified by
// re-reading the cited C# blocks and hand arithmetic.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use glam::Vec2;
    use tr_core::tile::Tile;

    fn world() -> World {
        World::new("t".into(), 200, 200)
    }

    fn proj(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(100.0 * 16.0, 50.0 * 16.0);
        p.vel = Vec2::new(8.0, 0.0);
        p.w = d.w;
        p.h = d.h;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.extra_updates = d.extra_updates;
        p
    }

    /// The rows carry the census slices (`Projectile.cs:7461-7473` /
    /// `:7508-7520`).
    #[test]
    fn w4_d80_the_dd2_shot_rows() {
        let d = proj_data(PHANTOM_PHOENIX).unwrap();
        assert_eq!(d.ai_style, 1);
        assert_eq!((d.w, d.h), (66.0, 66.0), "`:7463-7464`");
        assert_eq!(d.time_left, 300, "`:7468`");
        assert_eq!(d.penetrate, -1, "`:7471`");
        assert!(d.uses_local_immunity, "`:7470`");
        assert_eq!(d.local_hit_cooldown, 10, "`:7472`");
        assert!(d.ranged, "`:7469`");
        let d = proj_data(SKY_DRAGONS_FURY_SHOT).unwrap();
        assert_eq!(d.ai_style, 1);
        assert_eq!((d.w, d.h), (12.0, 12.0), "`:7510-7511`");
        assert_eq!(d.extra_updates, 1, "`:7517`");
        assert!(d.melee, "`:7514`");
        assert!(d.ignore_water, "`:7516`");
        assert!((d.scale - 0.6).abs() < 1e-6, "`:7519`");
    }

    /// The 706 one-shot latch: the first tick queues the play
    /// (`localAI[1] == 0`), the follow every tick; `localAI[1]` latches to 1.
    #[test]
    fn w4_d80_the_phoenix_plays_once_then_follows() {
        let w = world();
        let mut p = proj(PHANTOM_PHOENIX);
        step(&mut p, &w);
        assert_eq!(p.local_ai[1], 1.0, "the latch (`:55777`)");
        assert_eq!(
            p.wants_tracked,
            Some(ProjTrackedCue::HandlePlayFollow { style: "DD2_PhantomPhoenixShot", idx: 0, play: true }),
            "the play tick"
        );
        // The second tick: no play, the follow alone.
        p.wants_tracked = None;
        step(&mut p, &w);
        assert_eq!(
            p.wants_tracked,
            Some(ProjTrackedCue::HandlePlayFollow { style: "DD2_PhantomPhoenixShot", idx: 0, play: false }),
            "the follow tick"
        );
    }

    /// The 706 wet kill (`:55769-55773`): a liquid centre cell kills the shot.
    #[test]
    fn w4_d80_the_phoenix_dies_wet() {
        let mut w = world();
        let mut p = proj(PHANTOM_PHOENIX);
        let tx = (p.pos.x + p.w * 0.5) as i32 / 16;
        let ty = (p.pos.y + p.h * 0.5) as i32 / 16;
        let i = w.index(tx, ty);
        w.tiles[i] = Tile { liquid: 100, ..Default::default() };
        step(&mut p, &w);
        assert!(!p.active, "the wet kill");
    }

    /// The 706 frame machine (`:55837-55841`): `frame = frameCounter / 5`,
    /// the 40-tick wrap; the alpha fade 55 (`:55842-55847`).
    #[test]
    fn w4_d80_the_phoenix_frame_and_alpha() {
        let w = world();
        let mut p = proj(PHANTOM_PHOENIX);
        p.alpha = 255;
        for _ in 0..5 {
            step(&mut p, &w);
        }
        assert_eq!(p.frame_counter, 5);
        assert_eq!(p.frame, 1, "5 / 5 = 1");
        assert_eq!(p.alpha, 0, "255 - 5*55 = -20 → 0");
        // The 40-tick wrap.
        let mut q = proj(PHANTOM_PHOENIX);
        q.frame_counter = 39;
        step(&mut q, &w);
        assert_eq!(q.frame_counter, 0, "the 40 wrap");
        assert_eq!(q.frame, 0);
    }

    /// The 709 one-shot (`:54715-54718`): the `ai[1] == 0` tick queues the
    /// fire-and-forget play; the age counter kills at >= 30 (`:54720`).
    #[test]
    fn w4_d80_the_sky_dragons_fury_plays_once_and_ages() {
        let w = world();
        let mut p = proj(SKY_DRAGONS_FURY_SHOT);
        step(&mut p, &w);
        assert_eq!(
            p.wants_tracked,
            Some(ProjTrackedCue::OneShot("DD2_SkyDragonsFuryShot")),
            "the ai[1]==0 play"
        );
        assert_eq!(p.ai[1], 1.0);
        // The second tick: no play (ai[1] != 0).
        p.wants_tracked = None;
        step(&mut p, &w);
        assert_eq!(p.wants_tracked, None, "no re-play");
        assert_eq!(p.ai[1], 2.0);
        // The age kill at 30.
        let mut q = proj(SKY_DRAGONS_FURY_SHOT);
        q.ai[1] = 29.0;
        step(&mut q, &w);
        assert!(!q.active, "the >= 30 kill (`:54720-54723`)");
    }

    /// The 709 frame machine (`:54704-54713`): the 3-tick stride, wrap 3;
    /// the alpha fade 30 (`:54698-54702`).
    #[test]
    fn w4_d80_the_sky_dragons_fury_frame_and_alpha() {
        let w = world();
        let mut p = proj(SKY_DRAGONS_FURY_SHOT);
        p.alpha = 255;
        for _ in 0..3 {
            step(&mut p, &w);
        }
        assert_eq!(p.frame, 1, "3 ticks → frame 1");
        assert_eq!(p.frame_counter, 0);
        assert_eq!(p.alpha, 165, "255 - 3*30");
        // The frame wrap at 3.
        let mut q = proj(SKY_DRAGONS_FURY_SHOT);
        q.frame = 2;
        q.frame_counter = 2;
        step(&mut q, &w);
        assert_eq!(q.frame, 0, "the wrap 3 → 0");
    }

    /// The RNG contract: both arms draw nothing (they take no rng — the
    /// tracked plays' variation/pitch picks ride the audio-only statics at
    /// the drain, not here).
    #[test]
    fn w4_d80_the_dd2_shots_are_draw_free() {
        let w = world();
        let mut rng = tr_core::UnifiedRandom::new(7);
        let peek = rng.peek();
        let mut p = proj(PHANTOM_PHOENIX);
        step(&mut p, &w);
        let mut q = proj(SKY_DRAGONS_FURY_SHOT);
        step(&mut q, &w);
        assert_eq!(rng.peek(), peek, "zero Main.rand draws on the arm path");
    }
}
