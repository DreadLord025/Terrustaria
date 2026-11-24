//! The Frost Moon bosses' hostile-shot flight (S1-F-13c wiring) — the
//! per-type `AI_001` / aiStyle-14 / aiStyle-58 arms for the seven kinds the
//! Everscream / IceQueen / SantaNK1 machines queue
//! (`npc_ai_frost_boss*.rs` → `wants_hostile_shot(_ai)` → the pool's
//! data-carrying spawn, `projectile/data.rs`'s new rows).
//!
//! Without these arms the drained shots would ride the GENERIC aiStyle-1
//! flight (the 15-tick / 0.1 gravity) or the `_ => {}` no-op (aiStyle 58),
//! which is wrong for every one of them. The C# per-type branches:
//!
//! * **345 PineNeedle** (`Projectile.cs:57116-57128`) — aiStyle 1, `flag3`
//!   TRUE (not on the `:55947` skip list): the counter runs, the gravity
//!   arm pins `ai[0]` at 50 and adds 0.5/tick (NOT the generic 15/0.1).
//!   The `localAI[0]` first-tick latch (`:57118-57122`, `SoundID.Item1`
//!   skipped) rides.
//! * **348 FrostWave** (`:56246-56273`) — aiStyle 1, `flag3` FALSE
//!   (`:55985`): the counter rests, no gravity; the alpha/scale pulse
//!   cycles `ai[0]` 0→1→2. The `localAI[1]` latch (`:56248-56252`,
//!   `SoundID.Item8` skipped) rides.
//! * **349 FrostShard** (`:56221-56245`) — aiStyle 1, `flag3` FALSE
//!   (`:55986`): `frame = (int)ai[0]` (the drop's `ai0 = Next(5)`),
//!   `velocity.Y += 0.2f` EVERY tick, the alpha/scale pulse on `localAI[0]`.
//! * **350 Missile** (`:55130-55163`) — aiStyle 1, `flag3` FALSE
//!   (`:55987`): `alpha -= 100` clamp 0, the `ai[1]` latch (`:55158-55162`,
//!   `SoundID.Item42` skipped). The two dust rolls (`:55141-55156`, the
//!   `Next(3)`/`Next(10)`/`Next(4)`/`Next(5)` draws) are SKIPPED — the
//!   FLAMING_WOOD 325 precedent (`projectile_ai001_a.rs`, the identical
//!   `:55175-55189` arm); they ride the `UpdateProjectiles` stream, not
//!   the `UpdateNPCs` census this wiring pins (ANALYSIS).
//! * **346 Ornament** (`:24867-24897`) — aiStyle 14: the generic spear arm
//!   (`:25142-25160`) + `frame = (int)ai[1]`; the light (`:24877-24882`)
//!   and the `timeLeft == 1` death burst (`:24883-24896`, five 347
//!   Shrapnel spawns + rand) are deferred — the burst needs a pool-side
//!   spawn seam and the 347 row (ANALYSIS).
//! * **352 Spike** (`:24848-24866`) — aiStyle 14: the generic spear arm +
//!   the alpha/scale pulse (`localAI[1]` ±1, `alpha += 25*dir`,
//!   `scale += dir*0.01`), rand-free.
//! * **351 Present** (`:28906-28937`) — aiStyle 58: the whole arm,
//!   rand-free (the alpha fade, the `ai[0]` 0→1 flip at `velocity.Y >= 0`,
//!   the 0.1 gravity / 3f cap / 0.99 X friction, the atan2+1.57 rotation).
//!
//! Movement is NOT in `AI_001`/`AI_014` — it rides `HandleMovement` /
//! `UpdatePosition`; the port's established stand-in is
//! `ai_style1_flight`'s tail (the ±16 clamp, the default rotation, the
//! kill-on-tile integration) for aiStyle 1, and `ai_spear`'s tile-kill
//! scan + integrate for aiStyle 14. Both are replicated here verbatim
//! (cited) so the per-type gravity arms substitute cleanly.
//!
//! CODE-ONLY MODE (S1-F-13c wiring): the witness tests at the bottom are
//! WRITTEN NOT RUN (no compiler invoked); every expected value is
//! hand-derived from the cited C# lines.

use tr_core::World;

use crate::projectile::{
    Projectile, FROST_SHARD, FROST_WAVE, MISSILE, ORNAMENT_HOSTILE, PINE_NEEDLE_HOSTILE,
    PRESENT, SPIKE,
};

/// The seven Frost-boss shot kinds this module claims. The dispatch gate
/// in `ProjectilePool::update` routes ONLY these; every other kind keeps
/// its existing arm byte-identical.
pub(crate) const FROST_SHOT_KINDS: [u16; 7] = [
    PINE_NEEDLE_HOSTILE,
    ORNAMENT_HOSTILE,
    FROST_WAVE,
    FROST_SHARD,
    MISSILE,
    PRESENT,
    SPIKE,
];

pub(crate) fn handles(kind: u16) -> bool {
    FROST_SHOT_KINDS.contains(&kind)
}

/// One tick of the kind's flight. W4-g63 — the dust-free scratch entry
/// (the `ProjectilePool::update` precedent): the witnesses below run on
/// a scratch pool; the live sim threads the real pool + the
/// `UpdateProjectiles` stream through [`frost_shot_step_dust`].
pub(crate) fn frost_shot_step(p: &mut Projectile, world: &World) {
    let mut dust = tr_core::particles::DustPool::new();
    let ctx = tr_core::particles::SpawnCtx {
        game_menu: false,
        game_paused: false,
        generating: false,
        screen_position: glam::Vec2::ZERO,
        screen_size: glam::Vec2::new(
            world.width as f32 * tr_core::TILE_PX,
            world.height as f32 * tr_core::TILE_PX,
        ),
        max_dust_to_draw: 6000,
    };
    let mut rng = tr_core::UnifiedRandom::new(0);
    frost_shot_step_dust(p, world, &mut rng, &mut dust, &ctx);
}

/// W4-g63 — the full step with the caller's stream + the live dust pool:
/// the 350 Missile's `alpha == 0` dust window (`:55138-55157`) is
/// RE-ARMED; every other kind's stream stays untouched.
pub(crate) fn frost_shot_step_dust(
    p: &mut Projectile,
    world: &World,
    rng: &mut dyn tr_core::loot::DropRng,
    dust: &mut tr_core::particles::DustPool,
    dust_ctx: &tr_core::particles::SpawnCtx,
) {
    match p.kind {
        PINE_NEEDLE_HOSTILE => needle(p, world),
        FROST_WAVE => wave(p, world),
        FROST_SHARD => shard(p, world),
        MISSILE => missile(p, world, rng, dust, dust_ctx),
        ORNAMENT_HOSTILE => ornament(p, world),
        SPIKE => spike(p, world),
        PRESENT => present(p, world),
        _ => unreachable!("the dispatch gates on FROST_SHOT_KINDS"),
    }
}

/// The `localAI[0] == 0f → 1f` / `localAI[1]` first-tick latches (the sound
/// cues are skipped pool-wide; the latch state gates the cue to one play).
fn latch_local0(p: &mut Projectile) {
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
    }
}

/// The aiStyle-1 shared tail: the `flag3` counter, the per-kind gravity
/// (already applied by the caller), the default rotation
/// (`Projectile.cs:57750`), the +16 clamp (`:57764-57767`, `flag4` false
/// for every kind here) and the movement stand-in
/// (`ai_style1_flight`'s probe, `HandleMovement` `:19843-19853`).
fn finish_style1(p: &mut Projectile, world: &World, count_flight: bool) {
    if count_flight {
        p.ai[0] += 1.0;
    }
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    move_tile_collide(p, world);
}

/// The kill-on-tile integration (`ai_style1_flight`'s tail,
/// `projectile.rs`; `HandleMovement`'s default arm `:19843-19853`).
fn move_tile_collide(p: &mut Projectile, world: &World) {
    if p.tile_collide {
        let nx = p.pos.x + p.vel.x;
        let ny = p.pos.y + p.vel.y;
        let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
        let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
        if hit_x || hit_y {
            p.active = false;
            p.time_left = 0;
        } else {
            // S1-P-i57 — the `ai_style1_flight` swap (`:20441`'s wet arm via
            // `wet_velocity`; dry byte-identical).
            p.pos += crate::projectile_wet::wet_velocity(p, world);
        }
    } else {
        p.pos += p.vel;
    }
}

/// **345 PineNeedle** — `Projectile.cs:57116-57128` (the gravity arm) +
/// the `flag3` counter (`:56038-56041`, 345 is not on the skip list) +
/// the shared tail. The needle flies straight for 50 ticks then falls at
/// 0.5/tick.
fn needle(p: &mut Projectile, world: &World) {
    // `:57118-57122` — the first-tick latch (`SoundID.Item1` skipped).
    latch_local0(p);
    // `:56038-56041` — `flag3` true: the counter runs.
    p.ai[0] += 1.0;
    // `:57123-57127` — the gravity arm (mutually exclusive with the
    // generic `ai[0] >= 15f` tail, `:57568`).
    if p.ai[0] >= 50.0 {
        p.ai[0] = 50.0;
        p.vel.y += 0.5;
    }
    // The rotation/clamp/movement tail (no second counter increment).
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    move_tile_collide(p, world);
}

/// **348 FrostWave** — `Projectile.cs:56246-56273`. `flag3` false
/// (`:55985`): the counter rests, no gravity. The alpha/scale pulse cycles
/// `ai[0]` 0→1→2.
fn wave(p: &mut Projectile, world: &World) {
    // `:56248-56252` — the `localAI[1]` latch (`SoundID.Item8` skipped).
    if p.local_ai[1] == 0.0 {
        p.local_ai[1] = 1.0;
    }
    // `:56253-56272` — the pulse.
    if p.ai[0] == 0.0 || p.ai[0] == 2.0 {
        p.scale += 0.01;
        p.alpha -= 50;
        if p.alpha <= 0 {
            p.ai[0] = 1.0;
            p.alpha = 0;
        }
    } else if p.ai[0] == 1.0 {
        p.scale -= 0.01;
        p.alpha += 50;
        if p.alpha >= 255 {
            p.ai[0] = 2.0;
            p.alpha = 255;
        }
    }
    finish_style1(p, world, false);
}

/// **349 FrostShard** — `Projectile.cs:56221-56245`. `flag3` false
/// (`:55986`): `frame = (int)ai[0]`, `velocity.Y += 0.2f` every tick, the
/// alpha/scale pulse on `localAI[0]`.
fn shard(p: &mut Projectile, world: &World) {
    // `:56223-56224`.
    p.frame = p.ai[0] as i32;
    p.vel.y += 0.2;
    // `:56225-56244` — the pulse.
    if p.local_ai[0] == 0.0 || p.local_ai[0] == 2.0 {
        p.scale += 0.01;
        p.alpha -= 50;
        if p.alpha <= 0 {
            p.local_ai[0] = 1.0;
            p.alpha = 0;
        }
    } else if p.local_ai[0] == 1.0 {
        p.scale -= 0.01;
        p.alpha += 50;
        if p.alpha >= 255 {
            p.local_ai[0] = 2.0;
            p.alpha = 255;
        }
    }
    finish_style1(p, world, false);
}

/// **350 Missile** — `Projectile.cs:55130-55163`. `flag3` false
/// (`:55987`): `alpha -= 100` clamp 0, the `ai[1]` latch. W4-g63: the
/// `alpha == 0` dust window (`:55138-55157`) RE-ARMED on the live pool.
fn missile(
    p: &mut Projectile,
    world: &World,
    rng: &mut dyn tr_core::loot::DropRng,
    dust: &mut tr_core::particles::DustPool,
    dust_ctx: &tr_core::particles::SpawnCtx,
) {
    // `:55132-55136`.
    p.alpha -= 100;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:55137` — the light plot (skipped, no light layer).
    // `:55138-55157` — W4-g63: the two dust rolls at `alpha == 0`
    // RE-ARMED (the Next(3)/Next(10)/Next(4)/Next(5) draws ride the
    // pass's stream at their C# positions).
    if p.alpha == 0 {
        crate::projectile_trail_dust::santa_missile_dust(
            &crate::projectile_trail_dust::TrailSlot::of(p),
            rng, dust, dust_ctx, world,
        );
    }
    // `:55158-55162` — the `ai[1]` latch (`SoundID.Item42` skipped).
    if p.ai[1] == 0.0 {
        p.ai[1] = 1.0;
    }
    finish_style1(p, world, false);
}

/// The aiStyle-14 generic arm (`Projectile.cs:25142-25160`) + the tile-kill
/// scan (`:25704-25741`) + the integrate — `ai_spear`'s body replicated so
/// the 346/352 per-type hooks substitute cleanly. The `ai[0] > 5f` pin, the
/// 0.97 ground-slide friction, the 0.2 gravity and the `velocity.X * 0.1`
/// rotation.
fn spear_core(p: &mut Projectile, world: &World) {
    p.ai[0] += 1.0;
    if p.ai[0] > 5.0 {
        p.ai[0] = 5.0;
        if p.vel.y == 0.0 && p.vel.x != 0.0 {
            p.vel.x *= 0.97;
            if (-0.01..0.01).contains(&p.vel.x) {
                p.vel.x = 0.0;
            }
        }
        p.vel.y += 0.2;
    }
    p.rot += p.vel.x * 0.1;
    // `:25724-25741` — the (x+4, y+4) +8px centre probe vs solid, non-127
    // tiles; a hit kills.
    let cx = p.pos.x + 4.0;
    let cy = p.pos.y + 4.0;
    let tx0 = ((p.pos.x / 16.0) as i32 - 1).max(0);
    let tx1 = (((p.pos.x + p.w) / 16.0) as i32 + 2).min(world.width - 1);
    let ty0 = ((p.pos.y / 16.0) as i32 - 1).max(0);
    let ty1 = (((p.pos.y + p.h) / 16.0) as i32 + 2).min(world.height - 1);
    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if !t.active()
                || t.kind == 127
                || !tr_core::tiledata::TILE_SOLID.has(t.kind)
                || tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
            {
                continue;
            }
            let rx = tx as f32 * 16.0;
            let ry = ty as f32 * 16.0;
            if cx + 8.0 > rx && cx < rx + 16.0 && cy + 8.0 > ry && cy < ry + 16.0 {
                p.active = false;
                p.time_left = 0;
                return;
            }
        }
    }
    // S1-P-i57 — `:20441`'s wet swap on the aiStyle-14 integrate (the
    // `wet_velocity` helper; the row's `tileCollide = false` keeps it
    // un-scaled, dry byte-identical).
    p.pos += crate::projectile_wet::wet_velocity(p, world);
}

/// **346 Ornament** — `Projectile.cs:24867-24897` (the per-type hook) +
/// the aiStyle-14 generic arm. `frame = (int)ai[1]`; the light and the
/// `timeLeft == 1` death burst are deferred (module docs).
fn ornament(p: &mut Projectile, world: &World) {
    // `:24869-24873` — the `localAI[0]` latch (`SoundID.Item1` skipped).
    latch_local0(p);
    // `:24874` — `frame = (int)ai[1]` (the lob's `ai1 = Next(2)`).
    p.frame = p.ai[1] as i32;
    // `:24875-24882` — the light (skipped, no light layer).
    // `:24883-24896` — the death burst (deferred, module docs).
    spear_core(p, world);
}

/// **352 Spike** — `Projectile.cs:24848-24866` (the per-type hook) + the
/// aiStyle-14 generic arm. The alpha/scale pulse on `localAI[1]`, rand-free.
fn spike(p: &mut Projectile, world: &World) {
    // `:24850-24853` — the `localAI[1]` latch.
    if p.local_ai[1] == 0.0 {
        p.local_ai[1] = 1.0;
    }
    // `:24854-24865` — the pulse.
    p.alpha += (25.0 * p.local_ai[1]) as i32;
    if p.alpha <= 0 {
        p.alpha = 0;
        p.local_ai[1] = 1.0;
    } else if p.alpha >= 255 {
        p.alpha = 255;
        p.local_ai[1] = -1.0;
    }
    p.scale += p.local_ai[1] * 0.01;
    spear_core(p, world);
}

/// **351 Present** — the whole aiStyle-58 arm (`Projectile.cs:28906-28937`),
/// rand-free. The alpha fade, the `ai[0]` 0→1 flip at `velocity.Y >= 0`,
/// the 0.1 gravity / 3f cap / 0.99 X friction, the atan2+1.57 rotation.
/// Movement rides `UpdatePosition` (`:20445`, aiStyle 58 is not on the
/// `:20435` early-return list) — the raw integrate, `tileCollide` false.
fn present(p: &mut Projectile, world: &World) {
    // `:28908-28912` — the alpha fade.
    p.alpha -= 50;
    if p.alpha < 0 {
        p.alpha = 0;
    }
    // `:28913-28925` — the ai[0] == 0 state.
    if p.ai[0] == 0.0 {
        p.frame = 0;
        p.ai[1] += 1.0;
        if p.ai[1] > 30.0 {
            p.vel.y += 0.1;
        }
        if p.vel.y >= 0.0 {
            p.ai[0] = 1.0;
        }
    }
    // `:28926-28935` — the ai[0] == 1 state (a SIBLING if, runs the same
    // tick the flip above lands).
    if p.ai[0] == 1.0 {
        p.frame = 1;
        p.vel.y += 0.1;
        if p.vel.y > 3.0 {
            p.vel.y = 3.0;
        }
        p.vel.x *= 0.99;
    }
    // `:28936` — the rotation.
    p.rot = p.vel.y.atan2(p.vel.x) + 1.57;
    // `UpdatePosition` (`:20445`) — the raw integrate (tileCollide false).
    // S1-P-i57 — `:20441`'s wet swap via `wet_velocity` (the `:17476` gate
    // keeps it un-scaled; dry byte-identical).
    p.pos += crate::projectile_wet::wet_velocity(p, world);
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value is hand-derived from the cited C# lines.
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::proj_data;
    use glam::Vec2;
    use tr_core::TILE_PX;

    fn world() -> World {
        World::new("t".into(), 512, 64)
    }

    fn proj(kind: u16, vx: f32, vy: f32) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 255;
        p.pos = Vec2::new(10.0 * TILE_PX, 10.0 * TILE_PX);
        p.vel = Vec2::new(vx, vy);
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

    /// `:57123-57127` — the needle flies straight for 50 ticks (the
    /// counter pins at 50) then falls at 0.5/tick. Hand arithmetic: after
    /// 55 ticks ai[0] = 50, gravity fired on ticks 50..55 = 6 times →
    /// vel.y ≈ 6 × 0.5 = 3.0.
    #[test]
    fn the_needle_pins_at_fifty_and_falls_at_half() {
        let w = world();
        let mut p = proj(PINE_NEEDLE_HOSTILE, 2.0, 0.0);
        for _ in 0..55 {
            frost_shot_step(&mut p, &w);
        }
        assert_eq!(p.ai[0], 50.0, "the pin (`:57125`)");
        assert_eq!(p.local_ai[0], 1.0, "the first-tick latch (`:57120`)");
        assert!((p.vel.y - 6.0 * 0.5).abs() < 1e-4, "6 gravity ticks × 0.5");
        // The generic 15/0.1 gravity must NOT apply: at tick 20 the needle
        // still flies flat (its arm starts at 50).
        let mut q = proj(PINE_NEEDLE_HOSTILE, 2.0, 0.0);
        for _ in 0..20 {
            frost_shot_step(&mut q, &w);
        }
        assert_eq!(q.vel.y, 0.0, "no gravity before tick 50");
    }

    /// `:56246-56273` — the FrostWave rests the counter (flag3 false), no
    /// gravity; the pulse fades alpha 255→0 over 6 ticks (state 0→1 at tick
    /// 6), then oscillates 1↔2 (state 1→2 at tick 12, 2→1 at 18, 1→2 at 24).
    /// Hand arithmetic: after 24 ticks ai[0] = 2, alpha = 255.
    #[test]
    fn the_wave_rests_the_counter_and_pulses() {
        let w = world();
        let mut p = proj(FROST_WAVE, 3.0, 0.0);
        p.alpha = 255;
        for _ in 0..24 {
            frost_shot_step(&mut p, &w);
        }
        assert_eq!(p.ai[0], 2.0, "the pulse at state 2 on tick 24");
        assert_eq!(p.alpha, 255, "the state-2 entry clamps alpha (`:56270`)");
        assert_eq!(p.vel.y, 0.0, "no gravity — flag3 false (`:55985`)");
        assert_eq!(p.local_ai[1], 1.0, "the latch (`:56250`)");
        assert!((p.pos.x - (160.0 + 72.0)).abs() < 1e-3, "3 px/tick × 24");
    }

    /// `:56221-56245` — the shard falls at 0.2/tick every tick, frame rides
    /// ai[0] (the drop's Next(5) payload).
    #[test]
    fn the_shard_falls_every_tick_and_frames_from_ai0() {
        let w = world();
        let mut p = proj(FROST_SHARD, 1.0, 0.0);
        p.ai[0] = 3.0; // the drop's ai0 = Next(5)
        for _ in 0..10 {
            frost_shot_step(&mut p, &w);
        }
        assert_eq!(p.frame, 3, "frame = (int)ai[0] (`:56223`)");
        assert!((p.vel.y - 10.0 * 0.2).abs() < 1e-4, "0.2/tick × 10");
        assert_eq!(p.ai[0], 3.0, "the counter rests — flag3 false (`:55986`)");
    }

    /// `:55130-55163` — the missile fades alpha 255→0 over 3 ticks, rests
    /// the counter, latches ai[1].
    #[test]
    fn the_missile_fades_and_latches() {
        let w = world();
        let mut p = proj(MISSILE, 4.0, 0.0);
        for _ in 0..5 {
            frost_shot_step(&mut p, &w);
        }
        assert_eq!(p.alpha, 0, "255 − 100×3 clamped (`:55132-55136`)");
        assert_eq!(p.ai[1], 1.0, "the latch (`:55160`)");
        assert_eq!(p.ai[0], 0.0, "flag3 false (`:55987`)");
        assert_eq!(p.vel.y, 0.0, "no gravity");
    }

    /// `:24848-24866` — the spike's alpha/scale pulse: alpha climbs 25/tick
    /// to 255 then localAI[1] flips to −1.
    #[test]
    fn the_spike_pulses_alpha_and_scale() {
        let w = world();
        let mut p = proj(SPIKE, 2.0, 0.0);
        // 11 ticks: alpha 25×11 = 275 → clamps 255 at tick 11, dir flips.
        for _ in 0..11 {
            frost_shot_step(&mut p, &w);
        }
        assert_eq!(p.alpha, 255, "the clamp (`:24862`)");
        assert_eq!(p.local_ai[1], -1.0, "the flip (`:24863`)");
        assert_eq!(p.local_ai[0], 0.0, "the spike uses localAI[1], not [0]");
    }

    /// `:24867-24897` — the ornament frames from ai[1] (the lob's Next(2))
    /// and rides the spear gravity (0.2 after ai[0] > 5).
    #[test]
    fn the_ornament_frames_from_ai1_and_falls_like_a_spear() {
        let w = world();
        let mut p = proj(ORNAMENT_HOSTILE, 2.0, -3.0);
        p.ai[1] = 1.0; // the lob's ai1 = Next(2)
        for _ in 0..10 {
            frost_shot_step(&mut p, &w);
        }
        assert_eq!(p.frame, 1, "frame = (int)ai[1] (`:24874`)");
        assert_eq!(p.local_ai[0], 1.0, "the latch (`:24871`)");
        assert_eq!(p.ai[0], 5.0, "the spear pin (`:25147`)");
        // Gravity fired on ticks 6..10 = 5 times → vel.y = −3 + 5×0.2 = −2.
        assert!((p.vel.y - (-3.0 + 5.0 * 0.2)).abs() < 1e-4);
    }

    /// `:28906-28937` — the present: launched upward (vy < 0), the ai[0]==0
    /// state counts ai[1]; past 30 it adds 0.1 gravity; at vy >= 0 it flips
    /// to ai[0]==1 (the same tick), then falls at 0.1 capped 3 with 0.99 X
    /// friction.
    #[test]
    fn the_present_flips_at_apex_and_falls() {
        let w = world();
        let mut p = proj(PRESENT, 2.0, -4.0);
        // Tick 1: ai[1] = 1 (<= 30, no gravity), vy = −4 < 0 → stays state 0.
        frost_shot_step(&mut p, &w);
        assert_eq!(p.ai[0], 0.0);
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(p.vel.y, -4.0);
        // Run past the apex: vy climbs 0.1/tick after ai[1] > 30. From −4 it
        // reaches 0 around tick 70 (40 gravity ticks), flipping to state 1.
        // 75 ticks lands safely past the flip (no f32-epsilon edge).
        for _ in 0..74 {
            frost_shot_step(&mut p, &w);
        }
        assert_eq!(p.ai[0], 1.0, "the flip at vy >= 0 (`:28921-28924`)");
        assert_eq!(p.frame, 1, "the state-1 frame (`:28928`)");
        // A few more ticks: vy capped at 3, vx friction 0.99.
        let vx_before = p.vel.x;
        for _ in 0..40 {
            frost_shot_step(&mut p, &w);
        }
        assert!(p.vel.y <= 3.0 + 1e-4, "the 3f cap (`:28930-28933`)");
        assert!(p.vel.x < vx_before, "the 0.99 friction (`:28934`)");
        assert_eq!(p.alpha, 0, "the fade (`:28908-28912`)");
    }

    /// The dispatch claims exactly the seven kinds.
    #[test]
    fn the_gate_claims_the_seven_kinds() {
        for k in FROST_SHOT_KINDS {
            assert!(handles(k), "{k}");
        }
        for k in [0u16, 81, 180, 325, 347, 353, 448] {
            assert!(!handles(k), "{k}");
        }
    }
}
