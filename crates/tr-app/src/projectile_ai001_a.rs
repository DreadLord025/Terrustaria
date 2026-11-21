//! S1-D-5 slice A — `AI_001`'s per-type arms for the LOW-id rowed kinds
//! (`Projectile.cs:54002-57798`, the aiStyle-1 inline region). The port ran
//! ONE generic flight (`projectile.rs::ai_style1_flight`) for every rowed
//! aiStyle-1 kind; the C# has ~142 per-type branches. This module ports the
//! 14 lowest-id kinds whose branches need no NPC scan; slice B takes the
//! higher ids (≥ 449) disjointly — the kind set is [`SLICE_A_KINDS`] below
//! and is recorded in ROADMAP S1-D-5 / ANALYSIS "aiStyle-1 per-type arms".
//!
//! Kind set (all rowed in `projectile/data.rs`, C# `SetDefaults` aiStyle 1):
//! 14 MusketBall, 81 WoodenArrowHostile, 83 EyeLaser, 84 DeathLaser (the
//! Probe's laser, S1-D-3-wire-destroyer), 98 TrapDart,
//! 100 DestroyerLaser, 180 PirateArrow, 184 SuperDart, 248 FlowerPowPetal,
//! 259 GolemEyeBeam, 270 Skull, 325 FlamingWood, 389 MiniRetinaLaser,
//! 436 BrainScrambler, 438 RayGunner.
//!
//! C# structure preserved (the per-kind hooks above the flight, the shared
//! `flag3` counter switch `:55945-56041`, the gravity else-if chain
//! `:56888-57581`, the rotation tail chain `:57582-57752`, the ±16 clamp
//! `:57753-57768`): every kind here except 81 is on the `flag3 = false`
//! skip list, so its flight counter rests at 0 and the generic
//! `ai[0] >= 15f` gravity arm (`:57568-57579`) is unreachable — a laser,
//! bullet or dart flies exactly straight, as in vanilla. 81 keeps the
//! counter and takes its own 20-tick / 0.07 gravity arm (`:56888-56895`).
//!
//! RNG: NONE of the ported state paths draws `Main.rand` — every rand in
//! these kinds' C# blocks feeds dust/sound only (the skull's `:56079-56087`
//! dust burst, the flaming wood's `:55175-55189` dust, the scrambler's
//! light `:54173`), which the port skips pool-wide. The module takes no
//! rng parameter at all, so the `UpdateProjectiles` stream
//! (`Main.cs:17775-17777`) is untouched BY CONSTRUCTION for every kind.
//!
//! Movement is the port's documented tile-model stand-in (the same
//! destination-box probe `ai_style1_flight` uses, `HandleMovement`'s
//! default arm `:19843-19848`); the C# rotation tail runs BEFORE the clamp
//! (`:57582` then `:57753`), which this module keeps — the generic
//! `ai_style1_flight` keeps its own pre-existing order for the other kinds
//! (byte-identical, A/B-pinned by the tests below).
//!
//! `base.direction` reads (`:57599-57607`, `:57662-57670`) use the port's
//! velocity-sign stand-in — the same substitution `ai_book_of_skulls`
//! makes (the C# updates `direction` from `velocity.X`'s sign after each
//! sub-step's movement, `:17016-17026` + `AutomaticallyChangesDirection`
//! `:17281-17288`, which is true for aiStyle 1).

use glam::Vec2;
use tr_core::World;

use crate::projectile::{
    Projectile, BRAIN_SCRAMBLER, DESTROYER_LASER, EYE_LASER, FLOWER_POW_PETAL, FLAMING_WOOD,
    FLAMING_ARROW_HOSTILE, GOLEM_EYE_BEAM, MINI_RETINA_LASER, MUSKET_BALL, PIRATE_ARROW,
    PROBE_LASER, RAY_GUNNER, SKULL, TRAP_DART, TRAP_SUPERDART, WOODEN_ARROW_HOSTILE,
};

/// The slice-A kind set — slice B must take ids ≥ 449 disjointly
/// (ROADMAP S1-D-5). 185 (TrapSpikyBall) is deliberately NOT here: the
/// C# `SetDefaults` gives it aiStyle **14** (`Projectile.cs:2717-2727`);
/// its row carried aiStyle 1 until the S1-D-5/S1-D-4c data-row fix split it
/// onto its own style-14 arm.
pub(crate) const SLICE_A_KINDS: [u16; 16] = [
    MUSKET_BALL,          // 14
    WOODEN_ARROW_HOSTILE, // 81
    FLAMING_ARROW_HOSTILE, // 82
    EYE_LASER,            // 83
    PROBE_LASER,          // 84 (S1-D-3-wire-destroyer: the Probe's laser)
    TRAP_DART,            // 98
    DESTROYER_LASER,      // 100
    PIRATE_ARROW,         // 180
    TRAP_SUPERDART,       // 184
    FLOWER_POW_PETAL,     // 248
    GOLEM_EYE_BEAM,       // 259
    SKULL,                // 270
    FLAMING_WOOD,         // 325
    MINI_RETINA_LASER,    // 389
    BRAIN_SCRAMBLER,      // 436
    RAY_GUNNER,           // 438
];

/// The dispatch gate: only these kinds leave the generic `ai_arrow` arm.
pub(crate) fn handles(kind: u16) -> bool {
    SLICE_A_KINDS.contains(&kind)
}

/// The per-kind gravity arm of the `:56888-57581` else-if chain. Every
/// slice-A kind except 81 rests its counter (`flag3 = false`), so the
/// chain's generic `ai[0] >= 15f` tail is unreachable for them.
enum Gravity {
    /// `flag3 = false` (`:55947-56032`): `ai[0]` never increments, no
    /// gravity arm fires — straight flight.
    Rest,
    /// `:56888-56895` - `if (ai[0] >= 20f) { ai[0] = 20f;
    /// velocity.Y += 0.07f; }` (the hostile wooden arrow).
    HostileArrow,
    /// `:57568-57578` — the generic arrow arm: `if (ai[0] >= 15f) {
    /// ai[0] = 15f; velocity.Y += 0.1f; }` (`flag4` false — the 1006
    /// inversion is not a slice-A kind). The hostile FlamingArrow 82.
    GenericArrow,
}

/// The per-kind rotation tail of the `:57582-57752` chain.
enum Rot {
    /// `:57748-57751` — `rotation = Atan2(vy, vx) + 1.57f` (the literal
    /// 1.57, not π/2 — the C# spells the constant).
    Default,
    /// `:57586-57596` — the petal spins by `(|vx| + |vy|) * 0.05f`,
    /// signed by `velocity.X < 0f`.
    Petal,
    /// `:57597-57607` — `spriteDirection = direction`; the sprite faces
    /// the flight direction, mirrored when `direction < 0`.
    Skull,
    /// `:57662-57670` — `rotation = ToRotation() + π`, plus another π
    /// when `direction == -1`.
    Scrambler,
    /// `:57744-57747` — `rotation = ToRotation() - π/2`.
    FlamingWood,
}

/// `AI_001` for one slice-A kind: the per-type hooks in the C#'s own
/// order, then the shared counter/gravity/rotation/clamp tail, then the
/// movement stand-in. `owner` is the owner player's center (the pool's
/// `Main.player[owner]` channel) — only the skull's homing reads it.
pub(crate) fn ai001_slice_a(p: &mut Projectile, world: &World, owner: Option<(f32, f32)>) {
    match p.kind {
        // `:55950` case 14 — no per-type hook at all: the musket ball is
        // a straight bullet (the generic flight's gravity was the bug).
        MUSKET_BALL => finish(p, world, false, Gravity::Rest, Rot::Default),
        // `:55346-55350` — the first-tick sound latch: the C# plays
        // `SoundID.Item5` BEFORE setting `ai[1] = 1f` (S1-E-21h: the cue
        // rides the latch helper below), then the counter runs (`flag3`
        // true — 81 is not on the skip list) into the 20/0.07 gravity arm.
        WOODEN_ARROW_HOSTILE => {
            latch_ai1(p, 5); // Projectile.cs:55346-55350
            finish(p, world, true, Gravity::HostileArrow, Rot::Default);
        }
        // `:55346-55350` — the 81/82 shared latch arm (`SoundID.Item5`
        // SKIPPED, the `ai[1] = 1f` state rides). 82 is NOT on the flag3
        // skip list (`:55947-56032`) so the counter runs, and NOT on the
        // 81/91 gravity arm (`:56888`) — it takes the GENERIC arrow
        // gravity (`:57568-57578`: at 15 ticks `ai[0] = 15`,
        // `velocity.Y += 0.1f`), unlike the wooden twin's 20/0.07. The
        // ElfArcher's shot (S1-D-3 W-6). The per-tick flame dust
        // (`:17036-17040`, `Dust.NewDust` type 6) is a VISUAL with its
        // own draws — the projectile draw-pass is unported (ANALYSIS).
        FLAMING_ARROW_HOSTILE => {
            latch_ai1(p, 5); // Projectile.cs:55346-55350 (SoundID.Item5)
            finish(p, world, true, Gravity::GenericArrow, Rot::Default);
        }
        // `:55229-55233` — `ai[1] = 1f` + `SoundID.Item33` (S1-E-21h).
        EYE_LASER => {
            latch_ai1(p, 33); // Projectile.cs:55229-55233
            finish(p, world, false, Gravity::Rest, Rot::Default);
        }
        // `:55291-55295` — `ai[1] = 1f` + `SoundID.Item12` (S1-E-21h); 84 is
        // on the `flag3 = false` skip list (`:55956`) — straight flight.
        PROBE_LASER => {
            latch_ai1(p, 12); // Projectile.cs:55291-55295
            finish(p, world, false, Gravity::Rest, Rot::Default);
        }
        // `:55311-55315` — `ai[1] = 1f` + `SoundID.Item17` (S1-E-21h).
        TRAP_DART => {
            latch_ai1(p, 17); // Projectile.cs:55311-55315
            finish(p, world, false, Gravity::Rest, Rot::Default);
        }
        // `:55306-55310` — `ai[1] = 1f` + `SoundID.Item33` (S1-E-21h).
        DESTROYER_LASER => {
            latch_ai1(p, 33); // Projectile.cs:55306-55310
            finish(p, world, false, Gravity::Rest, Rot::Default);
        }
        // `:55351-55355` — the sound (`SoundID.Item11`) plays BEFORE the
        // latch in the C# (S1-E-21h: the helper keeps that order).
        PIRATE_ARROW => {
            latch_ai1(p, 11); // Projectile.cs:55351-55355
            finish(p, world, false, Gravity::Rest, Rot::Default);
        }
        // `:55321-55325` — `ai[1] = 1f` + `SoundID.Item17` (S1-E-21h).
        TRAP_SUPERDART => {
            latch_ai1(p, 17); // Projectile.cs:55321-55325
            finish(p, world, false, Gravity::Rest, Rot::Default);
        }
        // `:55356-55360` — the sound-then-latch pair (`SoundID.Item17`,
        // S1-E-21h); the spin tail is `:57586-57596`.
        FLOWER_POW_PETAL => {
            latch_ai1(p, 17); // Projectile.cs:55356-55360
            finish(p, world, false, Gravity::Rest, Rot::Petal);
        }
        GOLEM_EYE_BEAM => {
            // `:55239-55243` — `ai[1] = 1f` + `SoundID.Item33` (S1-E-21h).
            latch_ai1(p, 33);
            // `:56699-56709` — the alpha fade-in: `alpha -= 10`, clamp 0.
            if p.alpha > 0 {
                p.alpha -= 10;
            }
            if p.alpha < 0 {
                p.alpha = 0;
            }
            finish(p, world, false, Gravity::Rest, Rot::Default);
        }
        SKULL => skull(p, world, owner),
        FLAMING_WOOD => {
            // `:55164-55196` — `alpha -= 100` clamp 0; the light plot
            // (`:55171`) and the two dust rolls (`:55175-55189`, the
            // `Main.rand.Next(3)`/`Next(4)`/`Next(10)`/`Next(5)` draws)
            // are SKIPPED — dust-only, no state reads them.
            p.alpha -= 100;
            if p.alpha < 0 {
                p.alpha = 0;
            }
            // `:55191-55195` — `ai[1] = 1f` + `SoundID.Item42`: the
            // trackable custom cue has no legacy-Item facade (the D3
            // trackable pool, wave A) — DEFERRED (S1-E-21h), the latch
            // state rides alone.
            latch_ai1_no_cue(p);
            finish(p, world, false, Gravity::Rest, Rot::FlamingWood);
        }
        // `:55296-55300` — `ai[1] = 1f` + `SoundID.Item12` (S1-E-21h).
        MINI_RETINA_LASER => {
            latch_ai1(p, 12); // Projectile.cs:55296-55300
            finish(p, world, false, Gravity::Rest, Rot::Default);
        }
        BRAIN_SCRAMBLER => {
            // `:54150-54174` — the head block: the latch, the alpha fade,
            // the sprite direction and the 3-tick frame machine.
            latch_ai1(p, 12); // :54152-54156 (SoundID.Item12, S1-E-21h)
            p.alpha -= 40; // :54157-54161
            if p.alpha < 0 {
                p.alpha = 0;
            }
            // `:54162` — `spriteDirection = base.direction` (the
            // velocity-sign stand-in, module doc).
            p.sprite_direction = if p.vel.x < 0.0 { -1 } else { 1 };
            // `:54163-54172` — `frameCounter++`, at 3: `frame++`,
            // `frameCounter = 0`, wrap at 4. The light (`:54173`) SKIPPED.
            p.frame_counter += 1;
            if p.frame_counter >= 3 {
                p.frame += 1;
                p.frame_counter = 0;
                if p.frame >= 4 {
                    p.frame = 0;
                }
            }
            finish(p, world, false, Gravity::Rest, Rot::Scrambler);
        }
        RAY_GUNNER => {
            // `:54829-54832` — the light plot only (SKIPPED).
            // `:55254-55258` — `ai[1] = 1f` + `SoundID.Item12` (S1-E-21h).
            latch_ai1(p, 12);
            // `:55660-55671` — the speed-scaled fade: `num109 =
            // (float)Math.Sqrt(vx*vx + vy*vy)` (a DOUBLE sqrt cast back
            // to float), `alpha -= (byte)((double)num109 * 0.9)` while
            // positive, clamp 0. The byte cast truncates; speeds stay
            // under the ±16 clamp so the product never wraps 256.
            let speed = ((p.vel.x * p.vel.x + p.vel.y * p.vel.y) as f64).sqrt() as f32;
            if p.alpha > 0 {
                p.alpha -= (speed as f64 * 0.9) as u8 as i32;
            }
            if p.alpha < 0 {
                p.alpha = 0;
            }
            finish(p, world, false, Gravity::Rest, Rot::Default);
        }
        _ => unreachable!("the dispatch gates on SLICE_A_KINDS"),
    }
}

/// The `ai[1] == 0f → ai[1] = 1f` first-tick latch every laser/dart arm
/// in the `:55209-55365` chain shares. S1-E-21h: the arm's `SoundID.ItemN`
/// cue queues with the latch — `base.position`, the legacy id-2 style
/// play. The C# orders the play before the latch on 81/180/248 and after
/// it on the rest; the queue payload is identical either way (the drain
/// runs at the pass tail), so one helper keeps the pair.
fn latch_ai1(p: &mut Projectile, style: i32) {
    if p.ai[1] == 0.0 {
        p.wants_sound = Some((2, style, p.pos.x, p.pos.y, 0.0));
        p.ai[1] = 1.0;
    }
}

/// The cue-less latch — the arms whose C# cue is a deferred family
/// (the Flaming Wood's `SoundID.Item42` trackable, `:55191-55195`).
fn latch_ai1_no_cue(p: &mut Projectile) {
    if p.ai[1] == 0.0 {
        p.ai[1] = 1.0;
    }
}

/// The Skull 270 — the Skeletron head's expert spin shot
/// (`Projectile.cs:56056-56089` the homing, `:56657-56697` the
/// alpha/frame block, `:57597-57607` the rotation tail). `flag3 = false`
/// (`:55974`), so the counter rests and the generic gravity never fires;
/// the shot steers at the player only inside the `30 < ai[1] < 110`
/// window and accelerates 2%/tick below the speed 18.
fn skull(p: &mut Projectile, world: &World, owner: Option<(f32, f32)>) {
    // `:56058-56060` — `Player.FindClosest(Center, 1, 1)` then `ai[1]++`.
    // The port has one player: the owner center IS the closest player
    // (the pool's `Main.player[owner]` stand-in). No owner resolved →
    // no steer target (the C# always has a player; recorded in ANALYSIS).
    p.ai[1] += 1.0;
    // `:56061-56070` — the steer window: blend the velocity toward the
    // player at the weight 25, then re-normalise to the cached speed.
    if p.ai[1] < 110.0 && p.ai[1] > 30.0 {
        if let Some((ox, oy)) = owner {
            let speed = p.vel.length();
            let cx = p.pos.x + p.w * 0.5;
            let cy = p.pos.y + p.h * 0.5;
            let mut toward = Vec2::new(ox - cx, oy - cy);
            let len = toward.length();
            // XNA `Normalize()` on a zero vector is undefined; the port
            // keeps zero (a skull spawned on the player's center).
            if len > 0.0 {
                toward /= len;
            }
            toward *= speed;
            p.vel = (p.vel * 24.0 + toward) / 25.0;
            let len = p.vel.length();
            if len > 0.0 {
                p.vel = p.vel / len * speed;
            }
        }
    }
    // `:56071-56074` — `if (velocity.Length() < 18f) velocity *= 1.02f`.
    if p.vel.length() < 18.0 {
        p.vel *= 1.02;
    }
    // `:56075-56088` — the first-tick latch; `SoundID.Item8` queues with the
    // `localAI[0] = 1f` one-shot (S1-E-21h d76, `:56078` — `base.position`),
    // the 10× dust 5 burst stays SKIPPED (no rand draw survives).
    if p.local_ai[0] == 0.0 {
        p.local_ai[0] = 1.0;
        p.wants_sound = Some((2, 8, p.pos.x, p.pos.y, 0.0));
    }
    // `:56659-56675` — `alpha = 0` (the fade below it is a no-op for
    // 270), `frame++` wrapping at 2; the 2× dust 5 (`:56678-56685`)
    // SKIPPED.
    p.alpha = 0;
    p.frame += 1;
    if p.frame > 2 {
        p.frame = 0;
    }
    // 270 falls through the gravity chain to the generic `ai[0] >= 15f`
    // tail (`:57568`), unreachable with the resting counter.
    finish(p, world, false, Gravity::Rest, Rot::Skull);
}

/// The shared tail: the `flag3` counter (`:56038-56041`), the per-kind
/// gravity arm, the rotation tail (`:57582-57752`), the `flag8` clamp
/// (`:57753-57768` — only the +16 bound; the −16 bound is `flag4`'s,
/// kind 1006 only, not a slice-A kind), then the movement stand-in.
fn finish(p: &mut Projectile, world: &World, count_flight: bool, gravity: Gravity, rot: Rot) {
    // `Projectile.cs:56038-56041` — `if (flag3) ai[0]++`.
    if count_flight {
        p.ai[0] += 1.0;
    }
    match gravity {
        Gravity::Rest => {}
        // `Projectile.cs:56890-56894`.
        Gravity::HostileArrow => {
            if p.ai[0] >= 20.0 {
                p.ai[0] = 20.0;
                p.vel.y += 0.07;
            }
        }
        // `Projectile.cs:57568-57578` — the generic arrow gravity (the
        // hostile FlamingArrow 82; `flag4` false, so the +0.1 branch).
        Gravity::GenericArrow => {
            if p.ai[0] >= 15.0 {
                p.ai[0] = 15.0;
                p.vel.y += 0.1;
            }
        }
    }
    // The rotation tail chain, in the C#'s order BEFORE the clamp.
    let angle = p.vel.y.atan2(p.vel.x);
    match rot {
        // `Projectile.cs:57750`.
        Rot::Default => p.rot = angle + 1.57,
        // `Projectile.cs:57588-57595`.
        Rot::Petal => {
            let spin = (p.vel.x.abs() + p.vel.y.abs()) * 0.05;
            if p.vel.x < 0.0 {
                p.rot -= spin;
            } else {
                p.rot += spin;
            }
        }
        // `Projectile.cs:57599-57607` — the direction-sign stand-in
        // (module doc); `spriteDirection = direction` rides.
        Rot::Skull => {
            p.sprite_direction = if p.vel.x < 0.0 { -1 } else { 1 };
            p.rot = if p.vel.x < 0.0 {
                (-p.vel.y).atan2(-p.vel.x)
            } else {
                angle
            };
        }
        // `Projectile.cs:57664-57669`.
        Rot::Scrambler => {
            p.rot = angle + std::f32::consts::PI;
            if p.vel.x < 0.0 {
                p.rot += std::f32::consts::PI;
            }
        }
        // `Projectile.cs:57746`.
        Rot::FlamingWood => p.rot = angle - std::f32::consts::FRAC_PI_2,
    }
    // `Projectile.cs:57753-57768` — `flag8` is true for every slice-A
    // kind (only 577 opts out); `flag4` is false, so just the +16 bound.
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    // The movement: the port's tile-model stand-in, byte-identical to
    // `ai_style1_flight`'s probe (`HandleMovement`'s default arm
    // `Projectile.cs:19843-19848`; the blocked hit kills — the arrows
    // break on contact).
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

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value below is hand-derived from the cited C# lines;
// the RNG-order argument is by construction (the module takes no rng).
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{
        proj_data, FlailInputs, ProjectilePool, YoyoInputs, WOODEN_ARROW, SAUCER_LASER,
    };
    use tr_core::{TILE_PX, World};

    /// A tile-free world: the flight witnesses run in open air.
    fn world() -> World {
        World::new("t".into(), 512, 64)
    }

    /// A slot carrying its `SetDefaults` row slice (box, tileCollide,
    /// alpha), spawned at (160, 160) with the given velocity.
    fn proj(kind: u16, vx: f32, vy: f32) -> Projectile {
        let d = proj_data(kind).expect("the rowed kind");
        // Field-by-field on `Default` — the struct has private bookkeeping
        // fields (E0451 outside projectile.rs).
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
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

    fn step_pool(pool: &mut ProjectilePool, w: &mut World) {
        let mut rng = tr_core::tile_light::FastRandom::new(7);
        let mut npcs = Vec::new();
        pool.update(
            w,
            &mut npcs,
            None,
            None,
            None,
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut rng,
            0,
            false,
        );
    }

    /// `:55950` case 14 + `:56038-56041`: the musket ball's counter rests
    /// at 0, so the generic `ai[0] >= 15f` gravity (`:57568`) never fires.
    /// Hand arithmetic: 30 ticks at (2, 0) → vel stays (2, 0), ai[0] = 0,
    /// rot = atan2(0, 2) + 1.57 = 1.57 every tick (`:57750`).
    #[test]
    fn s1_d_5a_the_musket_ball_flies_straight_with_a_resting_counter() {
        let w = world();
        let mut p = proj(MUSKET_BALL, 2.0, 0.0);
        for _ in 0..30 {
            ai001_slice_a(&mut p, &w, None);
        }
        assert_eq!(p.ai[0], 0.0, "flag3 = false (`:55950`)");
        assert_eq!(p.vel.y, 0.0, "no gravity — the generic arm is unreachable");
        assert!((p.rot - 1.57).abs() < 1e-6);
        assert!((p.pos.x - (160.0 + 60.0)).abs() < 1e-4, "2 px/tick × 30");
    }

    /// `:55346-55350` latch + `:56890-56894`: the hostile arrow counts
    /// (`flag3` true), pins ai[0] at 20 and adds 0.07/tick from tick 20.
    /// Hand arithmetic: after 25 ticks ai[0] = 20 (pinned), gravity fired
    /// on ticks 20..25 = 6 times → vel.y ≈ 6 × 0.07 = 0.42.
    #[test]
    fn s1_d_5a_the_hostile_arrow_pins_at_twenty_and_falls_at_007() {
        let w = world();
        let mut p = proj(WOODEN_ARROW_HOSTILE, 2.0, 0.0);
        for _ in 0..25 {
            ai001_slice_a(&mut p, &w, None);
        }
        assert_eq!(p.ai[1], 1.0, "the first-tick latch (`:55349`)");
        assert_eq!(p.ai[0], 20.0, "the pin (`:56892`)");
        assert!(
            (p.vel.y - 6.0 * 0.07).abs() < 1e-5,
            "6 gravity ticks × 0.07, got {}",
            p.vel.y
        );
        // The plain arrow's 15/0.1 gravity must NOT apply: at tick 16
        // the hostile arrow still flies flat (its arm starts at 20).
        let mut q = proj(WOODEN_ARROW_HOSTILE, 2.0, 0.0);
        for _ in 0..16 {
            ai001_slice_a(&mut q, &w, None);
        }
        assert_eq!(q.vel.y, 0.0, "no gravity before tick 20");
    }

    /// S1-D-3 W-6 — the hostile FlamingArrow 82 (the ElfArcher's shot):
    /// the 81/82 shared latch (`:55346-55350`), the counter runs (82 is
    /// NOT on the flag3 skip list), and the GENERIC arrow gravity
    /// (`:57568-57578`: pin at 15, +0.1/tick) — NOT the wooden twin's
    /// 20/0.07. Hand arithmetic: after 20 ticks ai[0] = 15 (pinned),
    /// gravity fired on ticks 15..20 = 6 times → vel.y ≈ 6 × 0.1 = 0.6.
    #[test]
    fn s1_d_3_w6_the_flaming_arrow_hostile_pins_at_fifteen_and_falls_at_01() {
        let w = world();
        let mut p = proj(FLAMING_ARROW_HOSTILE, 2.0, 0.0);
        for _ in 0..20 {
            ai001_slice_a(&mut p, &w, None);
        }
        assert_eq!(p.ai[1], 1.0, "the first-tick latch (`:55349`)");
        assert_eq!(p.ai[0], 15.0, "the generic pin (`:57570`)");
        assert!(
            (p.vel.y - 6.0 * 0.1).abs() < 1e-5,
            "6 gravity ticks × 0.1, got {}",
            p.vel.y
        );
        // Before tick 15 the arrow flies flat (the generic arm's gate).
        let mut q = proj(FLAMING_ARROW_HOSTILE, 2.0, 0.0);
        for _ in 0..14 {
            ai001_slice_a(&mut q, &w, None);
        }
        assert_eq!(q.vel.y, 0.0, "no gravity before tick 15");
        // The row: 10x10, stock penetrate 1 / timeLeft 3600, arrow false.
        let d = proj_data(FLAMING_ARROW_HOSTILE).expect("the rowed kind");
        assert_eq!((d.w, d.h), (10.0, 10.0));
        assert_eq!(d.penetrate, 1);
        assert_eq!(d.time_left, 3600);
        assert!(!d.arrow, "the 82 arm never sets `arrow` (`:1625-1631`)");
    }

    /// The six latch-only lasers/darts (`:55229`/`:55311`/`:55306`/
    /// `:55351`/`:55321`/`:55296`): ai[1] latches to 1 on tick 1 and
    /// holds; the counter rests; 30 ticks at (2, 0) stay flat.
    #[test]
    fn s1_d_5a_the_laser_and_dart_latches_hold_and_fly_straight() {
        let w = world();
        for kind in [
            EYE_LASER,
            TRAP_DART,
            DESTROYER_LASER,
            PIRATE_ARROW,
            TRAP_SUPERDART,
            MINI_RETINA_LASER,
        ] {
            let mut p = proj(kind, 2.0, 0.0);
            // The row's extraUpdates would multiply sub-steps in the
            // pool; the direct arm call is one C# AI() invocation.
            ai001_slice_a(&mut p, &w, None);
            assert_eq!(p.ai[1], 1.0, "kind {kind}: the latch (`ai[1] = 1f`)");
            for _ in 0..29 {
                ai001_slice_a(&mut p, &w, None);
            }
            assert_eq!(p.ai[1], 1.0, "kind {kind}: the latch holds");
            assert_eq!(p.ai[0], 0.0, "kind {kind}: flag3 = false");
            assert_eq!(p.vel.y, 0.0, "kind {kind}: straight flight");
        }
    }

    /// `:56699-56709`: the eye beam fades 10/tick. Hand arithmetic:
    /// alpha 255 → after 10 ticks 155; after 30 ticks 0 (clamped).
    #[test]
    fn s1_d_5a_the_golem_eye_beam_fades_ten_per_tick() {
        let w = world();
        let mut p = proj(GOLEM_EYE_BEAM, 2.0, 0.0);
        p.alpha = 255; // the spawn-site override (the row keeps 0)
        for _ in 0..10 {
            ai001_slice_a(&mut p, &w, None);
        }
        assert_eq!(p.alpha, 155);
        assert_eq!(p.ai[1], 1.0, "the `:55239` latch");
        for _ in 0..20 {
            ai001_slice_a(&mut p, &w, None);
        }
        assert_eq!(p.alpha, 0, "255 − 300 clamps at 0");
        assert_eq!(p.vel.y, 0.0, "tileCollide = false, straight (`:55972`)");
    }

    /// `:57588-57595`: the petal spins (|vx|+|vy|)·0.05 per tick, signed
    /// by vx. Hand arithmetic: (2, 0) → +0.1/tick → rot 1.0 after 10;
    /// (−2, 0) → −0.1/tick → rot −1.0. The counter rests (`:55970`).
    #[test]
    fn s1_d_5a_the_flower_pow_petal_spins_by_its_speed() {
        let w = world();
        let mut p = proj(FLOWER_POW_PETAL, 2.0, 0.0);
        for _ in 0..10 {
            ai001_slice_a(&mut p, &w, None);
        }
        assert!((p.rot - 1.0).abs() < 1e-5, "10 × (2 × 0.05)");
        assert_eq!(p.ai[1], 1.0, "the `:55356` latch");
        assert_eq!(p.ai[0], 0.0);
        let mut q = proj(FLOWER_POW_PETAL, -2.0, 0.0);
        for _ in 0..10 {
            ai001_slice_a(&mut q, &w, None);
        }
        assert!((q.rot + 1.0).abs() < 1e-5, "the negative-x sign flips");
    }

    /// `:56056-56089`: the skull's speed boost and steer window. Hand
    /// arithmetic: start (10, 0); each tick with speed < 18 multiplies
    /// 1.02. 10·1.02^29 ≈ 17.758 < 18 → tick 30 boosts to ≈ 18.113 ≥ 18,
    /// so after 30 ticks the boost stops and ai[1] = 30 (not > 30, no
    /// steer yet). Tick 31 enters the window: the owner sits above and
    /// behind (the skull flew ~400 px right), so the blend
    /// (v·24 + toward)/25 re-normalised to the cached speed turns vy
    /// negative and keeps |v| — the assertions only need the sign and
    /// the magnitude, not the exact heading.
    #[test]
    fn s1_d_5a_the_skull_boosts_to_eighteen_then_steers_in_its_window() {
        let w = world();
        let mut p = proj(SKULL, 10.0, 0.0);
        p.alpha = 255; // the SetDefaults row (`:3558-3566`)
        let owner = Some((p.pos.x + p.w * 0.5, p.pos.y - 100.0));
        for _ in 0..30 {
            ai001_slice_a(&mut p, &w, owner);
        }
        assert_eq!(p.ai[1], 30.0, "ai[1]++ per tick (`:56060`)");
        assert_eq!(p.ai[0], 0.0, "flag3 = false (`:55974`)");
        assert_eq!(p.local_ai[0], 1.0, "the first-tick latch (`:56077`)");
        assert_eq!(p.alpha, 0, "the 270 alpha pin (`:56661`)");
        let expect = 10.0f32 * 1.02f32.powi(30);
        assert!(
            (p.vel.length() - expect).abs() < 1e-3,
            "10 × 1.02^30 ≈ {expect}, got {}",
            p.vel.length()
        );
        assert!(p.vel.y == 0.0, "no steer at ai[1] = 30 (strict > 30)");
        let speed_before = p.vel.length();
        ai001_slice_a(&mut p, &w, owner);
        assert_eq!(p.ai[1], 31.0);
        assert!(p.vel.y < 0.0, "the steer turns toward the owner above");
        assert!(
            (p.vel.length() - speed_before).abs() < 1e-3,
            "the re-normalise keeps the cached speed (`:56068-56069`)"
        );
        // The frame machine wraps 0→1→2→0 (`:56671-56675`): 31 ticks
        // from frame 0 → 31 % 3 = 1.
        assert_eq!(p.frame, 31 % 3);
    }

    /// `:55164-55196` + `:57746`: the flaming wood fades 100/tick and
    /// faces angle − π/2. Hand arithmetic: alpha 255 → 155 → 55 → 0;
    /// (2, 0) → rot = −π/2.
    #[test]
    fn s1_d_5a_the_flaming_wood_fades_fast_and_faces_minus_half_pi() {
        let w = world();
        let mut p = proj(FLAMING_WOOD, 2.0, 0.0);
        ai001_slice_a(&mut p, &w, None);
        assert_eq!(p.alpha, 155);
        ai001_slice_a(&mut p, &w, None);
        assert_eq!(p.alpha, 55);
        ai001_slice_a(&mut p, &w, None);
        assert_eq!(p.alpha, 0, "55 − 100 clamps at 0");
        assert_eq!(p.ai[1], 1.0, "the `:55191` latch");
        assert!(
            (p.rot + std::f32::consts::FRAC_PI_2).abs() < 1e-6,
            "atan2(0, 2) − π/2"
        );
        assert_eq!(p.vel.y, 0.0, "flag3 = false (`:55984`)");
    }

    /// `:54150-54174` + `:57664-57669`: the scrambler fades 40/tick,
    /// runs the 3-tick frame machine (wrap 4) and faces angle + π (+ π
    /// more when moving left). Hand arithmetic: alpha 255 → 0 after 7
    /// ticks (255 − 280); 12 ticks → frame 0, frameCounter 0 (4 frames
    /// × 3 ticks); (2, 0) → rot π; (−2, 0) → rot 3π (no wrap in the C#).
    #[test]
    fn s1_d_5a_the_brain_scrambler_fades_flaps_and_faces_backwards() {
        let w = world();
        let mut p = proj(BRAIN_SCRAMBLER, 2.0, 0.0);
        p.alpha = 255;
        for _ in 0..7 {
            ai001_slice_a(&mut p, &w, None);
        }
        assert_eq!(p.alpha, 0);
        assert_eq!(p.ai[1], 1.0, "the `:54152` latch");
        for _ in 0..5 {
            ai001_slice_a(&mut p, &w, None);
        }
        assert_eq!(p.frame, 0, "12 ticks = 4 frames, wrapped (`:54168`)");
        assert_eq!(p.frame_counter, 0);
        assert!((p.rot - std::f32::consts::PI).abs() < 1e-5, "angle + π");
        assert_eq!(p.sprite_direction, 1);
        let mut q = proj(BRAIN_SCRAMBLER, -2.0, 0.0);
        ai001_slice_a(&mut q, &w, None);
        assert!(
            (q.rot - 3.0 * std::f32::consts::PI).abs() < 1e-4,
            "atan2(0, −2) = π, + π + π = 3π"
        );
        assert_eq!(q.sprite_direction, -1);
    }

    /// `:55660-55671`: the ray gunner's fade is (byte)(speed·0.9)/tick.
    /// Hand arithmetic: (10, 0) → speed 10 → 9/tick; alpha 255 → 165
    /// after 10 ticks. The latch (`:55254`) holds; flight is straight.
    #[test]
    fn s1_d_5a_the_ray_gunner_fades_by_nine_at_speed_ten() {
        let w = world();
        let mut p = proj(RAY_GUNNER, 10.0, 0.0);
        p.alpha = 255;
        for _ in 0..10 {
            ai001_slice_a(&mut p, &w, None);
        }
        assert_eq!(p.alpha, 165, "255 − 10 × (byte)(10 × 0.9)");
        assert_eq!(p.ai[1], 1.0);
        assert_eq!(p.ai[0], 0.0, "flag3 = false (`:55994`)");
        assert_eq!(p.vel.y, 0.0);
    }

    /// The A/B seam witness, pool-level: a slice-A kind (14) takes the
    /// new arm through the dispatch (resting counter, flat flight) while
    /// the plain wooden arrow keeps the generic flight byte-identical
    /// (15 ticks → vel.y = 0.1, the pre-existing DW-3.2 pin) and a
    /// slice-B kind (449, the saucer laser) rides slice B's landed arm.
    #[test]
    fn s1_d_5a_the_seam_gates_only_the_slice_a_kinds() {
        // The musket ball: the old generic arm would have gravity after
        // 15 ticks; the new arm rests the counter.
        let mut pool = ProjectilePool::new();
        pool.spawn_with_data(MUSKET_BALL, 160.0, 160.0, 2.0, 0.0, 200, 0.0);
        let mut w = world();
        for _ in 0..30 {
            step_pool(&mut pool, &mut w);
        }
        let p = &pool.slots()[0];
        assert_eq!(p.ai[0], 0.0, "the seam routed 14 to the slice-A arm");
        assert_eq!(p.vel.y, 0.0);

        // The plain arrow: unchanged generic flight (the DW-3.2 pin).
        let mut arrow = ProjectilePool::new();
        arrow.spawn_with_data(WOODEN_ARROW, 160.0, 160.0, 2.0, 0.0, 200, 0.0);
        let mut w = world();
        for _ in 0..15 {
            step_pool(&mut arrow, &mut w);
        }
        assert!(
            (arrow.slots()[0].vel.y - 0.1).abs() < 1e-6,
            "the generic flight is byte-identical for the plain arrow"
        );

        // A slice-B kind (449 ≥ 449): now rides slice B's arm (landed —
        // the flag3 list rests its counter and the localAI[0] flight-time
        // ramp runs +3/tick, `:54794-54795`).
        let mut saucer = ProjectilePool::new();
        saucer.spawn_with_data(SAUCER_LASER, 160.0, 160.0, 2.0, 0.0, 200, 0.0);
        let mut w = world();
        for _ in 0..20 {
            step_pool(&mut saucer, &mut w);
        }
        assert_eq!(
            saucer.slots()[0].ai[0], 0.0,
            "449 rides slice B's resting counter (flag3, :55997)"
        );
        assert_eq!(
            saucer.slots()[0].local_ai[0], 100.0,
            "slice B's +3/tick flight-time ramp saturates at the 100 cap \
             (20 ticks x 2 sub-steps x 3 = 120 -> 100, :54794-54795; the \
             row's extraUpdates = 1)"
        );
        assert!(!handles(SAUCER_LASER), "the kind sets are disjoint");
    }

    /// S1-E-21h d76 (`:56078`): the skull's first-tick Item8 queues at
    /// base.position with the localAI[0] one-shot, once per life.
    #[test]
    fn s1_e_21h_d76_the_skull_first_tick_cue() {
        let w = world();
        let mut p = proj(SKULL, 10.0, 0.0);
        let (x0, y0) = (p.pos.x, p.pos.y);
        ai001_slice_a(&mut p, &w, None);
        assert_eq!(p.local_ai[0], 1.0, "the one-shot (`:56077`)");
        assert_eq!(
            p.wants_sound,
            Some((2, 8, x0, y0, 0.0)),
            "Item8 at the play-time position"
        );
        p.wants_sound = None;
        ai001_slice_a(&mut p, &w, None);
        assert_eq!(p.wants_sound, None, "one cue per life");
    }
}
