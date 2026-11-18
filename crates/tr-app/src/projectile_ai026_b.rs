//! AI_026 slice B (S1-D-4b-11) — the spinning boss-head pets of the
//! aiStyle-26 mega-family (`Projectile.cs:58102-65024`).
//!
//! # The slice boundary (verified against the C# type dispatch)
//!
//! The task's premise named "boomerangs/discs/paladin hammer" as the
//! higher-id AI_026 types — the decompiled dispatch says otherwise:
//! the boomerangs/discs are aiStyle **3** (`AI_003_Boomerang`,
//! `Projectile.cs:38866`, dispatched at `:23690-23693`; the port's
//! `ai_boomerang` already covers the family) and the Paladin's Hammer is
//! aiStyle **15** (`DefaultToFlail`, `:10753-10762`; the port's
//! `ai_flail`). The ROADMAP's S1-D-4b-8 row was corrected the same way
//! by slice A on 2026-11-18. Every type id appearing in `AI_026`'s body
//! (`:58102-65024`) is a pet, a minion or a Pal summon.
//!
//! Slice A (`task/S1-D-4b-10-melee-a`) claims the 14 early-return flyer
//! pets **175/197/198/199/211/380/774/815/817/886/892/894/901/1046**
//! (the five flight arms `:59120-60091`). This slice takes the two
//! HIGHER-id types with a self-contained early-return arm of their own:
//!
//! * **885** `ProjectileID.SkeletronPet` and
//! * **889** `ProjectileID.SkeletronPrimePet`
//!
//! — the shared boss-head spin arm `:60092-60253`, which returns before
//! the idle/walk/jump/attack machine (`:60254+`). Their `SetDefaults`
//! blocks are the 884-886 (`:8486-8495`) and 889-892 (`:8521-8530`)
//! ranges: 20x20, aiStyle 26, friendly, `penetrate = -1`,
//! `timeLeft *= 5` (3600 → 18000), `netImportant = true`. The keep-alive
//! flags are LIVE in the port: buffs 288/292 grant `pet_flags[885]` /
//! `pet_flags[889]` (`buff_effects_accessory.rs:62/66`,
//! `Player.cs:11105`/`:11121`).
//!
//! The rest of `AI_026` (the ground pets' idle machine `:60254-65024`,
//! the pygmies/spiders, the Pal summons 1094/1113, the 266 slime
//! minion's hop machine) stays unclaimed — the gate returns `false` for
//! every other kind and the pool's `_ => {}` fall-through keeps them
//! byte-identical.
//!
//! # RNG order (the golden rule)
//!
//! Per tick, in the C#'s evaluation order, the ONLY `Main.rand` draws
//! for these two types are the >2000 px teleport's 12 dusts
//! (`:60137-60142`): each `Dust.NewDust(SpeedX: 1f - Main.rand.NextFloat()
//! * 2f, SpeedY: 1f - Main.rand.NextFloat() * 2f, ...)` evaluates its
//! named arguments left-to-right — SpeedX's `NextFloat` THEN SpeedY's —
//! so the teleport consumes exactly **24 `NextFloat` draws** in that
//! order. The dust VISUALS are unported (no `Main.dust` layer), but the
//! 24 explicit draws are CONSUMED so the pass's `UpdateProjectiles`
//! stream stays aligned with the C# (slice A's precedent, ANALYSIS
//! "AI_026 slice B"). The dusts' INTERNAL rolls (`Dust.cs:114-190`) are
//! gated on the unported dust-pool occupancy and skip with the layer.
//! No head rand fires for these types (the pygmy `Next(2, 10)`
//! `:58919` is not this slice); the near/far/spin paths draw nothing.
//!
//! # Documented stand-ins
//!
//! * `Collision.SolidCollision` → [`crate::projectile::box_solid`] (the
//!   5-point approximation, ANALYSIS-documented since M4-PROJ-8).
//! * `Main.myPlayer == owner → netUpdate = true` (`:60145-60148`) →
//!   no-op (single-player port, the pool's standing convention).
//! * The head's `num = 95` override (`:58154-58169`) and the
//!   `flag2`/`flag3` direction band (`:59008-59026`) feed only the idle
//!   machine these types never reach — not ported (slice A's precedent).
//! * The petting flag (`flag9`, `:58118`) is read only by the Pal
//!   summons — not this slice.

use glam::Vec2;
use tr_core::World;
use tr_core::loot::DropRng;

use crate::player::Player;
use crate::projectile::{
    Projectile, SKELETRON_PET, SKELETRON_PRIME_PET, box_solid,
};

/// The slice-B type set — the two spinning boss-head pets. The dispatch
/// gate (`projectile.rs`'s `26 =>` arm) routes ONLY these; every other
/// aiStyle-26 kind keeps the pre-existing no-arm fall-through,
/// byte-identical.
pub(crate) const AI_026_B_KINDS: [u16; 2] = [SKELETRON_PET, SKELETRON_PRIME_PET];

/// The owner reads the arm shares — a snapshot of
/// `Main.player[this.owner]` taken once per tick (nothing in the arm
/// writes the player fields, so the C#'s re-reads are equivalent).
struct OwnerView {
    center: Vec2,
    /// `position.Y + height` — the grounded-reset bottom comparison
    /// (`:60106`).
    bottom: f32,
    /// `velocity.Y` — the grounded test (`:60106`).
    vel_y: f32,
    /// `Player.direction` (the port's `facing`).
    dir: i32,
}

impl OwnerView {
    fn capture(p: &Player) -> Self {
        let (cx, cy) = p.center();
        Self {
            center: Vec2::new(cx, cy),
            bottom: p.y + p.height,
            vel_y: p.vy,
            dir: p.facing,
        }
    }
}

/// The dispatch seam: `AI()`'s `else if (this.aiStyle == 26)
/// this.AI_026();` (`Projectile.cs:26416-26419`), gated to the slice-B
/// kinds. Returns `false` for every other kind — the caller keeps its
/// `_ => {}` fall-through, so non-slice routing is byte-identical.
///
/// On a slice-B tick this runs the observable head (the owner-inactive
/// kill `:58104-58108`, the keep-alive pin `:58669-58679`/`:58691-58701`,
/// the `flag11` pin `:58944-58947`), the shared boss-head spin arm
/// (`:60092-60253`), and `UpdatePosition`'s `position += velocity`
/// (`:20433-20446` — the arm sets `tileCollide = false` at `:60096`, so
/// `HandleMovement`'s collision half never runs; the `wet` branch folds
/// to the dry one, the slot carries no liquid-wet flags).
pub(crate) fn ai026_b_tick(
    p: &mut Projectile,
    world: &World,
    player: Option<&mut Player>,
    rng: &mut dyn DropRng,
) -> bool {
    if !AI_026_B_KINDS.contains(&p.kind) {
        return false;
    }
    // `:58104-58108` — `if (!Main.player[this.owner].active) { this.active
    // = false; return; }`. The port's `None` player is the unresolved /
    // inactive owner (the `ai_hook` dead-owner precedent).
    let Some(player) = player else {
        p.active = false;
        return true;
    };
    keep_alive(p, player);
    // `:58200-58202` + `:58944-58947` — `flag11 = ai[0] == -1 || -2` pins
    // `timeLeft = 2` (the despawn request the buff pass writes).
    if p.ai[0] == -1.0 || p.ai[0] == -2.0 {
        p.time_left = 2;
    }
    let owner = OwnerView::capture(player);
    boss_head_spin_arm(p, world, &owner, rng);
    // `UpdatePosition` (`:19858` → `:20445`): the dry `position +=
    // velocity` rides every tick — the arm's early returns leave
    // `AI_026`, not `Update`, so the integration still runs.
    p.pos += p.vel;
    true
}

/// The per-type pet keep-alive blocks — the 885 (`:58669-58679`) and the
/// 889 (`:58691-58701`): death clears `petFlagSkeletronPet` /
/// `petFlagSkeletronPrimePet`, then a STANDING flag pins `timeLeft = 2`.
/// The C# reads the flag AFTER the death write, so on the death tick the
/// cleared flag does not pin (the port's `pet_flags` array is indexed by
/// the pet projectile id, `player.rs:1284`).
fn keep_alive(p: &mut Projectile, player: &mut Player) {
    let idx = usize::from(p.kind);
    if player.dead {
        player.pet_flags[idx] = false;
    }
    if player.pet_flags[idx] {
        p.time_left = 2;
    }
}

/// The shared 885/889 arm (`Projectile.cs:60092-60253`) — the hover
/// chase at the `(-30 * direction, -65)` offset (the `dx -= 30 * dir`
/// at `:60101` puts the head BEHIND the owner), the >2000 px teleport
/// with its 24 consumed `NextFloat` draws, the ±accel steering with the
/// unconditional same-sign double boost, the far spin (frame 6,
/// `rotation += 0.5` wrapped at 2π) and the near walk (the 40-tick
/// `frameCounter` cycle, frames 0-5 at every 5th tick, the 885's
/// `localAI[0]` 0/1/2/1 ladder at counters 10/20/30/40 and the 889's
/// ±3 laser-eye accumulator capped at 100 / floored at 0).
fn boss_head_spin_arm(
    p: &mut Projectile,
    world: &World,
    owner: &OwnerView,
    rng: &mut dyn DropRng,
) {
    // `:60095-60105` — the parameter head.
    let mut accel = 0.15f32;
    p.tile_collide = false;
    let range = 150.0f32;
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let mut dx = owner.center.x - center.x;
    let mut dy = owner.center.y - center.y - 65.0;
    dx -= 30.0 * owner.dir as f32;
    let mut dist = (dx * dx + dy * dy).sqrt();
    let mut speed = 8.0f32;
    let raw_dist = dist;
    let teleport = dist > 2000.0;
    // `:60106-60109` — the grounded rise clamp.
    if dist < range
        && owner.vel_y == 0.0
        && p.pos.y + p.h <= owner.bottom
        && !box_solid(world, p.pos, p.w, p.h)
        && p.vel.y < -6.0
    {
        p.vel.y = -6.0;
    }
    // `:60110-60129` — the near damping / the far normalization.
    if dist < 10.0 {
        p.vel *= 0.9;
        if p.vel.length() < 0.5 {
            p.vel = Vec2::ZERO;
        }
        accel = 0.0;
    } else {
        if dist > range {
            accel = 0.2;
            speed = 12.0;
        }
        dist = speed / dist;
        dx *= dist;
        dy *= dist;
    }
    // `:60130-60149` — the teleport: 12 dusts × (SpeedX NextFloat, then
    // SpeedY NextFloat — the named arguments evaluate left-to-right),
    // the center snap and the velocity zero. The dust visuals skip with
    // the unported layer; the 24 explicit draws are CONSUMED (module
    // doc). `netUpdate` (`:60145-60148`) is the single-player no-op.
    if teleport {
        for _ in 0..12 {
            let _speed_x = 1.0 - rng.next_float() * 2.0;
            let _speed_y = 1.0 - rng.next_float() * 2.0;
        }
        p.pos = owner.center - Vec2::new(p.w * 0.5, p.h * 0.5);
        p.vel = Vec2::ZERO;
    }
    // `:60150-60181` — the steering. The same-sign second boost has NO
    // target-sign guard here (unlike the ground machine's `:60584+`) —
    // verbatim.
    if p.vel.x < dx {
        p.vel.x += accel;
        if p.vel.x < 0.0 {
            p.vel.x += accel;
        }
    }
    if p.vel.x > dx {
        p.vel.x -= accel;
        if p.vel.x > 0.0 {
            p.vel.x -= accel;
        }
    }
    if p.vel.y < dy {
        p.vel.y += accel;
        if p.vel.y < 0.0 {
            p.vel.y += accel;
        }
    }
    if p.vel.y > dy {
        p.vel.y -= accel;
        if p.vel.y > 0.0 {
            p.vel.y -= accel;
        }
    }
    // `:60182-60183` — the head faces AGAINST the owner; the sprite
    // faces with them.
    p.direction = -owner.dir;
    p.sprite_direction = -p.direction;
    // `:60184-60207` — the far spin (the test reads the PRE-scale
    // distance `num77`).
    if raw_dist >= range {
        p.rot += 0.5;
        if p.rot > std::f32::consts::PI * 2.0 {
            p.rot -= std::f32::consts::PI * 2.0;
        }
        p.frame = 6;
        p.frame_counter = 0;
        if p.kind == SKELETRON_PET {
            p.local_ai[0] = 0.0;
        }
        if p.kind == SKELETRON_PRIME_PET {
            p.local_ai[0] += 3.0;
            if p.local_ai[0] > 100.0 {
                p.local_ai[0] = 100.0;
            }
        }
        return;
    }
    // `:60208-60212` — the near rotation ease (the `< 0.05` test also
    // zeroes NEGATIVE rotations — verbatim quirk).
    p.rot *= 0.95;
    if p.rot < 0.05 {
        p.rot = 0.0;
    }
    // `:60213-60239` — the walk counter and the per-type accumulators.
    p.frame_counter += 1;
    if p.kind == SKELETRON_PET {
        match p.frame_counter {
            10 => p.local_ai[0] = 0.0,
            20 => p.local_ai[0] = 1.0,
            30 => p.local_ai[0] = 2.0,
            40 => p.local_ai[0] = 1.0,
            _ => {}
        }
    }
    if p.kind == SKELETRON_PRIME_PET {
        p.local_ai[0] -= 3.0;
        if p.local_ai[0] < 0.0 {
            p.local_ai[0] = 0.0;
        }
    }
    // `:60240-60251` — the 5-tick frame step (frames 0-5) and the 40-tick
    // counter wrap.
    if p.frame_counter % 5 == 0 {
        p.frame += 1;
        if p.frame > 5 {
            p.frame = 0;
        }
    }
    if p.frame_counter >= 40 {
        p.frame_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    //! Witness tests-as-code (S1-D-4b-11, code-only mode — written, NOT
    //! run this round). Each pins a C# behaviour with its citation; the
    //! RNG-order witness records the draw sequence on a counting stream.

    use super::*;
    use crate::projectile::{BABY_BIRD, SLIME_MINION, proj_data, proj_frames};

    /// A recording `DropRng` — the draw log pins the C#'s `Main.rand`
    /// order (the golden rule's RNG witness).
    #[derive(Default)]
    struct RecordRng {
        draws: Vec<String>,
    }
    impl DropRng for RecordRng {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push(format!("range({min},{max})"));
            min
        }
        fn next_float(&mut self) -> f32 {
            self.draws.push("float".into());
            0.5
        }
    }

    fn world() -> World {
        World::new("t".into(), 64, 64)
    }

    fn owner(w: &World) -> Player {
        let mut p = Player::spawn(w);
        p.x = 500.0;
        p.y = 500.0;
        p.vx = 0.0;
        p.vy = 0.0;
        p.facing = 1;
        p
    }

    fn pet(kind: u16, x: f32, y: f32) -> Projectile {
        let data = proj_data(kind).expect("slice-B row");
        Projectile {
            active: true,
            kind,
            owner: 0,
            pos: Vec2::new(x, y),
            w: data.w,
            h: data.h,
            friendly: true,
            time_left: data.time_left,
            penetrate: data.penetrate,
            max_penetrate: data.penetrate,
            tile_collide: data.tile_collide,
            ..Default::default()
        }
    }

    /// The hover point the arm chases: `playerCenter + (-30 * direction,
    /// -65)` (`:60099-60101` — the `num73 -= 30 * player.direction`
    /// subtracts), as the pet's top-left.
    fn hover_pos(p: &Player, kind: u16) -> Vec2 {
        let (cx, cy) = p.center();
        let d = proj_data(kind).unwrap();
        Vec2::new(cx - 30.0 * p.facing as f32 - d.w * 0.5, cy - 65.0 - d.h * 0.5)
    }

    /// The two rows carry the C# `SetDefaults` slice (`:8486-8495` /
    /// `:8521-8530`): 20x20, aiStyle 26, penetrate -1, timeLeft 18000,
    /// the stock tileCollide, netImportant true, ignoreWater false; the
    /// sheets are 10/11 frames (`Main.cs:8673`/`:8677`).
    #[test]
    fn the_two_rows_carry_the_csharp_set_defaults() {
        for kind in AI_026_B_KINDS {
            let d = proj_data(kind).expect("row");
            assert_eq!((d.w, d.h), (20.0, 20.0), "kind {kind} box");
            assert_eq!(d.ai_style, 26, "kind {kind} aiStyle");
            assert_eq!(d.penetrate, -1, "kind {kind} penetrate");
            assert_eq!(d.time_left, 18000, "kind {kind} timeLeft *= 5");
            assert!(d.tile_collide, "kind {kind} stock tileCollide");
            assert!(!d.minion && !d.sentry, "kind {kind} is a pet");
            assert!(d.net_important, "kind {kind} netImportant");
            assert!(!d.ignore_water, "kind {kind} ignoreWater");
        }
        assert_eq!(proj_frames(SKELETRON_PET), 10);
        assert_eq!(proj_frames(SKELETRON_PRIME_PET), 11);
    }

    /// The gate routes ONLY the two — the 266 slime minion (the rest of
    /// AI_026) and a non-26 kind return false and nothing mutates, RNG
    /// untouched.
    #[test]
    fn the_gate_leaves_non_slice_kinds_byte_identical() {
        let w = world();
        let mut p = owner(&w);
        for kind in [SLIME_MINION, BABY_BIRD] {
            let mut slot = pet(kind, 480.0, 480.0);
            let before = slot.clone();
            let mut rng = RecordRng::default();
            assert!(!ai026_b_tick(&mut slot, &w, Some(&mut p), &mut rng));
            assert_eq!(slot.pos, before.pos);
            assert_eq!(slot.vel, before.vel);
            assert_eq!(slot.time_left, before.time_left);
            assert!(rng.draws.is_empty(), "no rand for a non-slice kind");
        }
    }

    /// `:58104-58108` — an inactive owner kills the pet.
    #[test]
    fn an_absent_owner_kills_the_pet() {
        let w = world();
        let mut slot = pet(SKELETRON_PET, 480.0, 480.0);
        let mut rng = RecordRng::default();
        assert!(ai026_b_tick(&mut slot, &w, None, &mut rng));
        assert!(!slot.active);
    }

    /// `:58669-58679`/`:58691-58701` — a standing flag pins timeLeft = 2;
    /// death clears the flag FIRST, so the death tick does not pin.
    #[test]
    fn the_keep_alive_pins_and_the_death_clears_first() {
        let w = world();
        let mut p = owner(&w);
        let pos = hover_pos(&p, SKELETRON_PET);
        let mut slot = pet(SKELETRON_PET, pos.x, pos.y);
        p.pet_flags[usize::from(SKELETRON_PET)] = true;
        let mut rng = RecordRng::default();
        assert!(ai026_b_tick(&mut slot, &w, Some(&mut p), &mut rng));
        assert_eq!(slot.time_left, 2, "the standing flag pins");
        slot.time_left = 18000;
        p.dead = true;
        assert!(ai026_b_tick(&mut slot, &w, Some(&mut p), &mut rng));
        assert!(!p.pet_flags[usize::from(SKELETRON_PET)], "death clears");
        assert_eq!(slot.time_left, 18000, "the cleared flag does not pin");
    }

    /// `:58944-58947` — `ai[0] == -1 || -2` pins timeLeft = 2.
    #[test]
    fn the_despawn_request_pins_the_lifetime() {
        let w = world();
        let mut p = owner(&w);
        let pos = hover_pos(&p, SKELETRON_PRIME_PET);
        for state in [-1.0f32, -2.0] {
            let mut slot = pet(SKELETRON_PRIME_PET, pos.x, pos.y);
            slot.ai[0] = state;
            let mut rng = RecordRng::default();
            assert!(ai026_b_tick(&mut slot, &w, Some(&mut p), &mut rng));
            assert_eq!(slot.time_left, 2, "ai[0] = {state}");
        }
    }

    /// `:60184-60207` — beyond 150 px the head spins: frame 6, the +0.5
    /// rotation wrapped at 2π, the 885's accumulator reset and the 889's
    /// +3 capped at 100. The early return skips the walk counter.
    #[test]
    fn the_far_head_spins_and_caps_the_laser_accumulator() {
        let w = world();
        let mut p = owner(&w);
        let hover = hover_pos(&p, SKELETRON_PET);
        let mut rng = RecordRng::default();
        let mut s885 = pet(SKELETRON_PET, hover.x + 200.0, hover.y);
        s885.local_ai[0] = 2.0;
        s885.rot = std::f32::consts::PI * 2.0 - 0.1;
        assert!(ai026_b_tick(&mut s885, &w, Some(&mut p), &mut rng));
        assert_eq!(s885.frame, 6);
        assert_eq!(s885.frame_counter, 0);
        assert_eq!(s885.local_ai[0], 0.0, "the 885 resets");
        assert!(
            (s885.rot - (std::f32::consts::PI * 2.0 - 0.1 + 0.5
                - std::f32::consts::PI * 2.0))
                .abs()
                < 1e-4,
            "the spin wraps at 2π"
        );
        let hover = hover_pos(&p, SKELETRON_PRIME_PET);
        let mut s889 = pet(SKELETRON_PRIME_PET, hover.x + 200.0, hover.y);
        s889.local_ai[0] = 99.0;
        assert!(ai026_b_tick(&mut s889, &w, Some(&mut p), &mut rng));
        assert_eq!(s889.local_ai[0], 100.0, "the 889 caps at 100");
        assert!(rng.draws.is_empty(), "the spin draws no rand");
    }

    /// `:60213-60251` — at the hover point the walk runs: the 885's
    /// `localAI[0]` ladder 0/1/2/1 at counters 10/20/30/40, frames 0-5
    /// stepping every 5th tick, the counter wrapping at 40.
    #[test]
    fn the_skeletron_pet_walks_the_forty_tick_cycle() {
        let w = world();
        let mut p = owner(&w);
        let pos = hover_pos(&p, SKELETRON_PET);
        let mut slot = pet(SKELETRON_PET, pos.x, pos.y);
        let mut rng = RecordRng::default();
        let mut ladder = Vec::new();
        for tick in 1..=40 {
            assert!(ai026_b_tick(&mut slot, &w, Some(&mut p), &mut rng));
            // Re-pin the hover point: the arm integrates the (zero)
            // velocity, so the pet stays put — but keep the fixture
            // explicit.
            slot.pos = pos;
            if matches!(tick, 10 | 20 | 30 | 40) {
                ladder.push(slot.local_ai[0]);
            }
            if tick == 5 {
                assert_eq!(slot.frame, 1, "the 5th tick steps the frame");
            }
            if tick == 25 {
                assert_eq!(slot.frame, 5, "frame 5 at tick 25");
            }
            if tick == 30 {
                assert_eq!(slot.frame, 0, "frame 6 wraps to 0 at tick 30");
            }
        }
        assert_eq!(ladder, vec![0.0, 1.0, 2.0, 1.0], "the 0/1/2/1 ladder");
        assert_eq!(slot.frame_counter, 0, "the counter wraps at 40");
        assert_eq!(slot.frame, 2, "frames 1/2 on ticks 35/40");
        assert!(rng.draws.is_empty(), "the walk draws no rand");
    }

    /// `:60232-60239` — the 889's near walk decays the laser accumulator
    /// by 3 per tick, floored at 0.
    #[test]
    fn the_prime_pet_decays_the_laser_accumulator() {
        let w = world();
        let mut p = owner(&w);
        let pos = hover_pos(&p, SKELETRON_PRIME_PET);
        let mut slot = pet(SKELETRON_PRIME_PET, pos.x, pos.y);
        slot.local_ai[0] = 7.0;
        let mut rng = RecordRng::default();
        for expected in [4.0f32, 1.0, 0.0, 0.0] {
            assert!(ai026_b_tick(&mut slot, &w, Some(&mut p), &mut rng));
            slot.pos = pos;
            assert_eq!(slot.local_ai[0], expected);
        }
    }

    /// `:60130-60149` — the >2000 px teleport consumes exactly 24
    /// `NextFloat` draws (12 dusts × SpeedX-then-SpeedY), snaps the
    /// center to the owner and zeroes the velocity.
    #[test]
    fn the_teleport_consumes_twenty_four_floats_in_order() {
        let w = world();
        let mut p = owner(&w);
        let hover = hover_pos(&p, SKELETRON_PET);
        let mut slot = pet(SKELETRON_PET, hover.x + 2500.0, hover.y);
        slot.vel = Vec2::new(5.0, 5.0);
        let mut rng = RecordRng::default();
        assert!(ai026_b_tick(&mut slot, &w, Some(&mut p), &mut rng));
        assert_eq!(rng.draws.len(), 24, "12 dusts × 2 NextFloat");
        assert!(rng.draws.iter().all(|d| d == "float"), "all NextFloat");
        let (cx, cy) = p.center();
        // The center snap, then the post-arm integration of the zeroed
        // velocity (the steering adds accel toward the hover offset).
        assert!((slot.pos.x + slot.w * 0.5 - cx).abs() < 1.0);
        assert!((slot.pos.y + slot.h * 0.5 - cy).abs() < 1.0);
        assert!(slot.vel.length() < 1.0, "the velocity was zeroed");
    }

    /// `:60106-60109` — the grounded rise clamp: near, owner grounded,
    /// pet bottom above the owner's, no solid tile, rising faster than
    /// -6 → the rise is clamped BEFORE the near damping (×0.9). The
    /// fixture sits AT the hover point (dist 0 < 10) so the damping
    /// branch runs; the steering's accel is 0 there.
    #[test]
    fn the_grounded_rise_clamps_before_the_damping() {
        let w = world();
        let mut p = owner(&w);
        let pos = hover_pos(&p, SKELETRON_PET);
        let mut slot = pet(SKELETRON_PET, pos.x, pos.y);
        slot.vel = Vec2::new(0.0, -10.0);
        let mut rng = RecordRng::default();
        assert!(ai026_b_tick(&mut slot, &w, Some(&mut p), &mut rng));
        // -10 → clamp -6 → damping ×0.9 = -5.4 (accel 0 inside 10 px).
        assert!(
            (slot.vel.y - (-5.4)).abs() < 1e-4,
            "clamp then damp: {}",
            slot.vel.y
        );
    }

    /// `:60150-60165` — the steering's same-sign second boost has no
    /// target-sign guard (unlike the ground machine's `:60584+`): a
    /// velocity still moving WITH its own sign after the first accel
    /// step gets the second step too.
    #[test]
    fn the_steering_double_boosts_without_a_target_guard() {
        let w = world();
        let mut p = owner(&w);
        let hover = hover_pos(&p, SKELETRON_PET);
        // 50 px right of the hover point: dist 50 ∈ [10, 150), accel
        // 0.15, target dx = -8 * 50/50... normalized to speed 8.
        let mut slot = pet(SKELETRON_PET, hover.x + 50.0, hover.y);
        slot.vel = Vec2::new(8.2, 0.0);
        let mut rng = RecordRng::default();
        assert!(ai026_b_tick(&mut slot, &w, Some(&mut p), &mut rng));
        // dx = -8: 8.2 > -8 → -= 0.15 → 8.05 > 0 → -= 0.15 → 7.9.
        assert!((slot.vel.x - 7.9).abs() < 1e-4, "vel.x = {}", slot.vel.x);
        // `:60182-60183` — the head faces against the owner.
        assert_eq!(slot.direction, -1);
        assert_eq!(slot.sprite_direction, 1);
    }
}

