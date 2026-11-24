//! W4-d2 — the aiStyle-11 light pets: the Shadow Orb (18) and the three
//! fairies (72/86/87). `AI()`'s `else if (this.aiStyle == 11)`
//! (`Projectile.cs:24371-24500`), the style's whole body.
//!
//! Census verdicts (the C# the port now matches):
//!
//! * **The fairy head** (`flag5 = type == 72 || 86 || 87`, `:24373-24410`):
//!   the sprite flips AGAINST the X velocity (`> 0 → -1`, `< 0 → 1`),
//!   `rotation = velocity.X * 0.1f` (a SET, not an add), the 4-tick frame
//!   cycle (`frameCounter >= 4 → frame++`, `frame >= 4 → 0`) and the
//!   1-in-6 dust (`Next(6) == 0`, types 56/73/74, alpha 200, scale 0.8).
//!   The orb's else arm spins `rotation += 0.02f` (`:24413`).
//! * **The keep-alive pin** (`:24415-24439`): the AFFIRMATIVE flag read —
//!   `blueFairy`/`redFairy`/`greenFairy`/`lightOrb` true pins
//!   `timeLeft = 2` (the `AI_026` keep-alive's exact shape,
//!   `projectile_ai026_a::keep_alive`): the buff pass re-grants the flag
//!   every tick, so the pin holds the pet alive; the tick the buff lapses
//!   the flag stays false and the 2-tick life runs out. The port reads
//!   `Player.pet_flags[kind]` — the accessory pass's flag seam
//!   (`buff_effects_accessory.rs:247`, `Player.cs:11286-11297` /
//!   `:11008`).
//! * **The follow block** (`:24440-24495`): owner dead → `Kill()`; else
//!   the owner-center seek — speed 3 (fairies 6), the 800-px teleport,
//!   the 70-px rest band (fairies 50; the orb's controlUp/controlDown
//!   shifts the target ±40 Y / −6 X and narrows the band to 4), and the
//!   fairy-only `velocity.Length() > num89` cap to the band overshoot
//!   (`SafeNormalize(Zero) * num89`, `:24486-24489`). Inside the band the
//!   velocity zeroes (`:24493`'s chained assign).
//! * **The movement**: the rows keep `tileCollide = false`
//!   (`:1002`/`:1530`/`:1682`/`:1696`), so `HandleMovement`'s collision
//!   half never runs and `UpdatePosition` integrates `position +=
//!   velocity` (`:20445` — aiStyle 11 is not on the skip list `:20435`).
//!   `AutomaticallyChangesDirection` (`:17283`) does NOT exclude 11: the
//!   direction follows `velocity.X`'s sign after the move (`:17016-
//!   17026`), ported at the arm's tail (the same-sub-step order).
//!
//! RNG by construction:
//! - The fairy head consumes exactly ONE `Next(6)` per tick (`:24395`);
//!   on a hit the `Dust.NewDust` call's INTERNAL rolls are pool-occupancy
//!   gated (`Dust.cs:114-152`) and skip with the unported dust layer —
//!   the `projectile_ai026_a` wisp-dust precedent (its module doc), NOT
//!   the empress six-draw form. The `velocity *= 0.3f` / shader writes
//!   (`:24407-24408`) are dust-visual state, skipped with it.
//! - The orb arm draws NOTHING.
//!
//! CODE-ONLY MODE (W4-d2): the witness tests at the bottom are WRITTEN
//! NOT RUN (no compiler invoked on this branch); verified by re-reading
//! the cited C# blocks and hand arithmetic.
//!
//! Deliberate skips (ANALYSIS "The aiStyle-11 light pets"): the dust
//! visuals, `netUpdate`, the wet movement half (`ignoreWater = true` on
//! every row — the C# wet flags never set), the owner's `cLight` shader
//! read (draw state).

use glam::Vec2;
use tr_core::loot::DropRng;

use crate::player::Player;
use crate::projectile::{safe_normalize, Projectile, BLUE_FAIRY, GREEN_FAIRY, PINK_FAIRY, SHADOW_ORB};

/// The aiStyle-11 kinds (`Projectile.cs:993-1007`/`:1522-1535`/
/// `:1674-1701` — the style's whole SetDefaults census).
pub(crate) const LIGHT_PET_KINDS: [u16; 4] = [SHADOW_ORB, BLUE_FAIRY, PINK_FAIRY, GREEN_FAIRY];

/// One aiStyle-11 tick: the AI body (`Projectile.cs:24371-24500`) plus
/// the free-move integration and the direction flip. `player` is the
/// owner snapshot (`Main.player[this.owner]` — the C# reads it
/// unconditionally); `None` kills the orphan (the `ai_drill` guard).
pub(crate) fn light_pet_step(p: &mut Projectile, player: Option<&Player>, rng: &mut dyn DropRng) {
    let Some(pl) = player else {
        // No owner resolved — the C# cannot read `Main.player[owner]`;
        // the port's orphan guard (the ai026_a dead-owner precedent).
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    };
    // `:24373` — `flag5 = type == 72 || type == 86 || type == 87`.
    let fairy = matches!(p.kind, BLUE_FAIRY | PINK_FAIRY | GREEN_FAIRY);
    if fairy {
        // `:24376-24383` — the sprite flips AGAINST the velocity sign.
        if p.vel.x > 0.0 {
            p.sprite_direction = -1;
        } else if p.vel.x < 0.0 {
            p.sprite_direction = 1;
        }
        // `:24384` — `rotation = velocity.X * 0.1f` (a set).
        p.rot = p.vel.x * 0.1;
        // `:24385-24394` — the 4-tick frame cycle.
        p.frame_counter += 1;
        if p.frame_counter >= 4 {
            p.frame += 1;
            p.frame_counter = 0;
        }
        if p.frame >= 4 {
            p.frame = 0;
        }
        // `:24395-24409` — the 1-in-6 dust gate. The explicit Next(6)
        // rides the pass's stream; the NewDust internals skip with the
        // dust layer (module doc).
        if rng.next_range(0, 6) == 0 {
            // num81 = 56 / 73 (86) / 74 (87) — the dust type, visual only.
        }
    } else {
        // `:24413` — the orb's spin.
        p.rot += 0.02;
    }
    // `:24415-24439` — the keep-alive pin (the AFFIRMATIVE flag read —
    // the buff pass re-grants `pet_flags[kind]` every tick; a lapsed
    // buff leaves it false and the 2-tick life runs out).
    if pl.pet_flags[usize::from(p.kind)] {
        p.time_left = 2;
    }
    // `:24440-24499` — the follow block / the dead-owner kill.
    if pl.dead {
        // `:24498` — `this.Kill()`.
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        return;
    }
    // `:24442-24446` — num83 = 3f (fairies 6f).
    let speed = if fairy { 6.0 } else { 3.0 };
    // `:24447` — vector11 = the projectile centre (float halves).
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    // `:24448-24449` — the owner centre: `(float)(width / 2)` is the
    // C# INTEGER division of the int box (20/2 = 10, 42/2 = 21).
    let mut dx = pl.x + (pl.width as i32 / 2) as f32 - center.x;
    let mut dy = pl.y + (pl.height as i32 / 2) as f32 - center.y;
    // `:24450-24451` — num86 = 800 (the teleport), num87 = 70 (the band).
    let teleport_range = 800.0;
    let mut band = 70.0;
    if p.kind == SHADOW_ORB {
        // `:24452-24466` — the orb's controlUp/controlDown target shift.
        if pl.control_up {
            dy = pl.y - 40.0 - center.y;
            dx -= 6.0;
            band = 4.0;
        } else if pl.control_down {
            dy = pl.y + pl.height + 40.0 - center.y;
            dx -= 6.0;
            band = 4.0;
        }
    }
    if fairy {
        // `:24467-24470` — the fairies' wide band.
        band = 50.0;
    }
    // `:24471-24472` — num88 = sqrt(dx² + dy²) (the C# computes it twice
    // into the same local; one sqrt is the identical value).
    let mut dist = (dx * dx + dy * dy).sqrt();
    if dist > teleport_range {
        // `:24473-24477` — the teleport onto the owner centre, the box
        // halves INTEGER-divided (`(float)(base.width / 2)`).
        p.pos.x = pl.x + (pl.width as i32 / 2) as f32 - (p.w as i32 / 2) as f32;
        p.pos.y = pl.y + (pl.height as i32 / 2) as f32 - (p.h as i32 / 2) as f32;
    } else if dist > band {
        // `:24478-24490` — the normalized seek at num83, the fairy-only
        // overshoot cap to num89 = dist − band.
        let overshoot = dist - band;
        dist = speed / dist;
        dx *= dist;
        dy *= dist;
        p.vel.x = dx;
        p.vel.y = dy;
        if fairy && p.vel.length() > overshoot {
            p.vel = safe_normalize(p.vel) * overshoot;
        }
    } else {
        // `:24491-24494` — inside the band both axes zero.
        p.vel.x = 0.0;
        p.vel.y = 0.0;
    }
    // `UpdatePosition` (`:20445`): the free move — the rows keep
    // `tileCollide = false`, `ignoreWater = true` (the wet branch folds
    // dry, module doc).
    p.pos += p.vel;
    // `AutomaticallyChangesDirection` (`:17283` — 11 not excluded) +
    // `:17016-17026`: the direction follows the X sign AFTER the move.
    p.direction = if p.vel.x < 0.0 { -1 } else { 1 };
}

// =====================================================================
// Witness tests — WRITTEN, NOT RUN (code-only mode: no cargo/rustc).
// Every expected value is hand-derived from the cited C# lines; the RNG
// order is asserted against a UnifiedRandom mirror (the rabid-roll
// mirror pattern, buff_effects_misc.rs).
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::projectile::{proj_data, proj_frames};
    use tr_core::{UnifiedRandom, World};

    /// A scripted stream: every draw lands in `draws` in order.
    struct Recorder {
        draws: Vec<(i32, i32)>,
        next: i32,
    }
    impl DropRng for Recorder {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push((min, max));
            self.next
        }
        fn next_float(&mut self) -> f32 {
            panic!("the light pets draw no floats");
        }
    }

    fn pet(kind: u16) -> Projectile {
        let d = proj_data(kind).expect("the light-pet row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = kind;
        p.owner = 0;
        p.pos = Vec2::new(320.0, 320.0);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.tile_collide = d.tile_collide;
        p.alpha = d.alpha;
        p.scale = d.scale;
        p
    }

    fn owner() -> Player {
        let w = World::new("t".into(), 512, 256);
        let mut pl = Player::spawn(&w);
        pl.x = 320.0;
        pl.y = 320.0;
        pl
    }

    #[test]
    fn the_light_pet_rows_match_set_defaults() {
        // The style census: every rowed aiStyle-11 kind is a light pet.
        assert_eq!(LIGHT_PET_KINDS.len(), 4);
        for k in LIGHT_PET_KINDS {
            assert_eq!(proj_data(k).unwrap().ai_style, 11);
        }
        // `Projectile.cs:993-1007` — the orb: 32x32, style 11, pen −1,
        // life ×5, no tile collide, alpha 150, scale 0.8, light 0.9.
        let d = proj_data(SHADOW_ORB).unwrap();
        assert_eq!((d.w, d.h), (32.0, 32.0));
        assert_eq!(d.ai_style, 11);
        assert_eq!(d.penetrate, -1);
        assert_eq!(d.time_left, 18000);
        assert!(!d.tile_collide);
        assert_eq!(d.alpha, 150);
        assert_eq!(d.scale, 0.8);
        assert_eq!(d.light, 0.9);
        assert!(d.net_important && d.ignore_water);
        // `:1522-1535`/`:1674-1701` — the fairies: 18x18, NO alpha line.
        for k in [BLUE_FAIRY, PINK_FAIRY, GREEN_FAIRY] {
            let d = proj_data(k).unwrap();
            assert_eq!((d.w, d.h), (18.0, 18.0));
            assert_eq!(d.ai_style, 11);
            assert_eq!(d.penetrate, -1);
            assert_eq!(d.time_left, 18000);
            assert!(!d.tile_collide);
            assert_eq!(d.alpha, 0, "the fairies keep the stock alpha (:779)");
            assert_eq!(d.scale, 0.8);
            assert_eq!(d.light, 0.9);
            assert!(d.net_important && d.ignore_water);
            assert_eq!(proj_frames(k), 4, "Main.cs:8578-8580");
        }
        assert_eq!(proj_frames(SHADOW_ORB), 1, "no projFrames[18] override");
    }

    #[test]
    fn the_fairy_frame_cycle_wraps_at_four() {
        // `:24385-24394` — frame advances every 4 ticks, wraps 4 → 0.
        let mut p = pet(BLUE_FAIRY);
        let pl = owner();
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        for tick in 1..=16 {
            light_pet_step(&mut p, Some(&pl), &mut rng);
            let expect_frame = (tick / 4) % 4;
            assert_eq!(p.frame, expect_frame, "tick {tick}");
            assert_eq!(p.frame_counter, tick % 4);
        }
    }

    #[test]
    fn the_fairy_head_flips_the_sprite_and_sets_the_rotation() {
        // `:24376-24384` — vel.x > 0 → sprite −1; rotation = vx·0.1.
        let mut p = pet(PINK_FAIRY);
        p.vel = Vec2::new(5.0, 0.0);
        let pl = owner();
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert_eq!(p.sprite_direction, -1);
        // The follow block rewrites the velocity, but the rotation was
        // set from the PRE-follow velocity (the C# order).
        assert!((p.rot - 0.5).abs() < 1e-6, "rotation = 5·0.1, got {}", p.rot);
        p.vel = Vec2::new(-5.0, 0.0);
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert_eq!(p.sprite_direction, 1);
    }

    #[test]
    fn the_orb_spins_and_draws_nothing() {
        // `:24413` — rotation += 0.02; the orb arm has NO rand gate.
        let mut p = pet(SHADOW_ORB);
        let pl = owner();
        let mut rng = Recorder { draws: Vec::new(), next: 0 };
        for _ in 0..10 {
            light_pet_step(&mut p, Some(&pl), &mut rng);
        }
        assert!((p.rot - 0.2).abs() < 1e-5, "10 × 0.02, got {}", p.rot);
        assert!(rng.draws.is_empty(), "the orb draws nothing");
    }

    #[test]
    fn the_fairy_dust_gate_mirrors_the_unified_stream() {
        // `:24395` — ONE Next(6) per fairy tick, in tick order; the
        // mirror consumes the identical stream (the rabid-roll pattern).
        let mut p = pet(GREEN_FAIRY);
        let pl = owner();
        let mut rng = UnifiedRandom::new(1234);
        let mut mirror = UnifiedRandom::new(1234);
        let mut hits = 0;
        for _ in 0..60 {
            let gate = rng.next_range(0, 6);
            let expect = mirror.next_range(0, 6);
            assert_eq!(gate, expect);
            if gate == 0 {
                hits += 1;
            }
        }
        assert_eq!(rng.peek(), mirror.peek(), "RNG streams stay aligned");
        // The arm itself consumes exactly one (0,6) per tick.
        let mut rec = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, Some(&pl), &mut rec);
        assert_eq!(rec.draws, vec![(0, 6)]);
        let _ = hits;
    }

    #[test]
    fn the_affirmative_flag_pins_the_life_and_the_dead_owner_kills() {
        // `:24417-24419` — blueFairy TRUE pins timeLeft = 2 (the
        // AI_026 keep-alive shape); `:24498` — dead → Kill().
        let mut p = pet(BLUE_FAIRY);
        let mut pl = owner();
        pl.pet_flags[usize::from(BLUE_FAIRY)] = true;
        p.time_left = 900;
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert_eq!(p.time_left, 2, "the flag pins the 2-tick life");
        assert!(p.active);
        pl.pet_flags[usize::from(BLUE_FAIRY)] = false;
        p.time_left = 900;
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert_eq!(p.time_left, 900, "no flag → no pin (the buff-lapsed death)");
        pl.dead = true;
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert!(!p.active && p.time_left == 0);
        assert_eq!(p.kill_time_left, 900, "Kill()'s num capture (:70649)");
    }

    #[test]
    fn the_orb_flag_reads_the_light_orb_seam() {
        // `:24436-24439` — type 18 && lightOrb → the same pin.
        let mut p = pet(SHADOW_ORB);
        let mut pl = owner();
        pl.pet_flags[usize::from(SHADOW_ORB)] = true;
        p.time_left = 500;
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert_eq!(p.time_left, 2);
    }

    #[test]
    fn the_far_pet_teleports_onto_the_owner() {
        // `:24473-24477` — dist > 800 → position = owner centre − box/2.
        let mut p = pet(BLUE_FAIRY);
        let pl = owner(); // centre (330, 341)
        p.pos = Vec2::new(330.0 + 900.0, 341.0); // 900 px right
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, Some(&pl), &mut rng);
        // The teleport lands at (330−9, 341−9) = (321, 332); the velocity
        // is UNTOUCHED by the teleport arm (the C# writes position only),
        // so the integration adds the stock zero.
        assert_eq!(p.pos, Vec2::new(321.0, 332.0));
        assert_eq!(p.vel, Vec2::ZERO);
    }

    #[test]
    fn the_mid_range_seek_normalizes_to_the_family_speed() {
        // `:24478-24485` — 70 < dist ≤ 800 → velocity = delta·(speed/dist).
        let mut p = pet(BLUE_FAIRY);
        let pl = owner(); // centre (330, 341)
        // Centre at (330+100, 341): dist 100 (> band 50, < 800).
        p.pos = Vec2::new(421.0, 332.0);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, Some(&pl), &mut rng);
        // delta = (−100, 0), speed 6 → vel (−6, 0); overshoot 50 > 6 so
        // the cap does not fire. Integration: pos.x 421 − 6 = 415.
        assert_eq!(p.vel, Vec2::new(-6.0, 0.0));
        assert_eq!(p.pos, Vec2::new(415.0, 332.0));
        assert_eq!(p.direction, -1, "the X sign flips the direction");
        // The orb seeks at 3 with the 70-px band.
        let mut q = pet(SHADOW_ORB);
        q.pos = Vec2::new(414.0, 325.0); // centre (430, 341), dist 100
        light_pet_step(&mut q, Some(&pl), &mut rng);
        assert_eq!(q.vel, Vec2::new(-3.0, 0.0));
    }

    #[test]
    fn the_fairy_overshoot_caps_at_the_band_remainder() {
        // `:24486-24489` — fairy && |vel| > dist − 50 → normalize·(dist−50).
        let mut p = pet(PINK_FAIRY);
        let pl = owner();
        // Centre 53 px right of the owner: dist 53, band 50 → cap 3 (< 6).
        p.pos = Vec2::new(374.0, 332.0);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert!((p.vel.x + 3.0).abs() < 1e-4, "capped to −3, got {}", p.vel.x);
        assert!(p.vel.y.abs() < 1e-6);
    }

    #[test]
    fn the_rest_band_zeroes_the_velocity() {
        // `:24491-24494` — dist ≤ band → both axes zero.
        let mut p = pet(GREEN_FAIRY);
        let pl = owner();
        p.pos = Vec2::new(351.0, 332.0); // centre (360, 341), dist 30 ≤ 50
        p.vel = Vec2::new(4.0, 4.0);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert_eq!(p.vel, Vec2::ZERO);
        assert_eq!(p.pos, Vec2::new(351.0, 332.0), "no move at rest");
    }

    #[test]
    fn the_orb_control_shifts_the_target_and_narrows_the_band() {
        // `:24452-24466` — controlUp: target = (ownerCentre.X − 6,
        // owner.Y − 40), band 4. A spot 50 px below the SHIFTED target is
        // out of the 4-px band (the seek fires) while sitting inside the
        // unshifted 70-px band (rest without the control).
        let mut p = pet(SHADOW_ORB);
        let mut pl = owner();
        pl.control_up = true;
        // Centre (324, 330): shifted target (324, 280) → dist 50 > 4.
        p.pos = Vec2::new(308.0, 314.0);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert_eq!(p.vel, Vec2::new(0.0, -3.0), "the shifted target seeks up");
        // Without the control the same centre is ~12.5 px from the owner
        // centre — inside the 70-px band, so the velocity zeroes.
        pl.control_up = false;
        p.pos = Vec2::new(308.0, 314.0);
        p.vel = Vec2::new(2.0, 0.0);
        light_pet_step(&mut p, Some(&pl), &mut rng);
        assert_eq!(p.vel, Vec2::ZERO, "the unshifted band rests the orb");
    }

    #[test]
    fn the_orphan_guard_kills_the_unowned_pet() {
        let mut p = pet(BLUE_FAIRY);
        let mut rng = Recorder { draws: Vec::new(), next: 1 };
        light_pet_step(&mut p, None, &mut rng);
        assert!(!p.active && p.time_left == 0);
        assert!(rng.draws.is_empty(), "the guard draws nothing");
    }
}
