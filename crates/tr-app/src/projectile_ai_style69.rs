//! W4-d7 (aiStyle-69) — the Flairon's recall head (404), `Projectile.AI`'s
//! `else if (this.aiStyle == 69)` block (`Projectile.cs:30240-30303`), its
//! two hit latches (`Damage_PVE_Inner` `:14166-14170` / `Damage_PVP`
//! `:14709-14713`) and its `HandleMovement` blocked-move arm (the
//! `:19512-19571` aiStyle-3/13/69/109 group's 69 half), ported 1:1 as PURE
//! machines (code-only mode: no compiler/tests were run on this branch;
//! the witness tests below are written, not executed).
//!
//! ## Census (research/26e row 69)
//!
//! ONE vanilla member: Flairon 404 (`ProjectileID.cs:1362`, `SetDefaults`
//! `Projectile.cs:4847-4856`: 26×26, friendly, `penetrate = -1`,
//! `alpha = 255`, melee). NOT the port's `FLAIRON` const (1058 =
//! FlaironFlail, the aiStyle-15 flail BODY, data.rs:613): item 2611
//! Flairon shoots **404** (`Item.cs:25528` — the port's itemdata row
//! already carries `projectile: Some(404)`), item 5526 FlaironFlail shoots
//! 1058 (`Item.cs:43534`). The module landed UNWIRED (the d5 /
//! `npc_ai_style25` precedent) and is now WIRED (the W4-d7 rows+wiring
//! round): the 404 `proj_data` row + the dispatch's aiStyle-69 arm route
//! here. The weapon fires 404 outbound; the FIRST NPC hit (or tile block)
//! latches `ai[0] = 1` and the head recalls to the owner, spawning 405
//! bubbles on the way back.
//!
//! ## The AI machine (verbatim, `:30240-30303`)
//!
//! * `:30242-30243` — `v = owner.Center − Center`;
//!   `rotation = v.ToRotation() − 1.57f` (the f32 literal) BEFORE the
//!   dead check.
//! * `:30244-30248` — owner dead: `Kill()` + return.
//! * `:30249-30263` — `SetDummyItemTime(10)`, `ChangeDir(v.X < 0 ? 1 : −1)`
//!   + `direction` the same, `itemRotation = (v * −1 * direction)
//!   .ToRotation()`, `spriteDirection = !(v.X > 0) ? 1 : −1` (the X == 0
//!   quirk: direction −1 but spriteDirection +1 — kept).
//! * `:30264-30267` — the arm latch: `ai[0] == 0 && |v| > 400` → 1.
//! * `:30268-30292` — the recall (`ai[0]` 1 or 2): `|v| > 1500` → Kill +
//!   return; `|v| > 600` → state 2; `tileCollide = false`; speed 20
//!   (state 2: 40); `velocity = Normalize(v) * speed`; `|v| < speed` →
//!   the arrival Kill + return.
//! * `:30293-30297` — `ai[1]++` every tick; `> 5` → `alpha = 0`.
//! * `:30298-30302` — `(int)ai[1] % 4 == 0` + local owner: the 405 bubble
//!   spawn — `(−v).SafeNormalize(0) * (Next(45,65) * 0.1f)` rotated by
//!   `(NextDouble() − 0.5) * 1.5707963705062866` (the f64 seam,
//!   `bow_launch::rotated_by`), at Centre, `this.damage`/`this.knockBack`,
//!   `ai0 = −10f`.
//!
//! Movement: the row keeps `tileCollide = true` until the recall clears
//! it; `AutomaticallyChangesDirection` EXCLUDES 69 (`:17283`) — the arm
//! owns the direction. A blocked move reaches the `:19512` group (NOT the
//! kill-on-tile default): [`flairon_tile_hit`] latches `ai[0] = 1` +
//! netUpdate with NO velocity flip (the flip is the 3/109-only arm
//! `:19564-19568`); `UpdatePosition`'s shared integrate (`:20445`, 69 is
//! not in the `:20435` skip list) is the wiring caller's.
//!
//! ## RNG contract
//!
//! The ONLY draws are the local-owner every-4th-tick bubble spawn's
//! `Next(45,65)` speed THEN one `NextDouble` jitter (`:30300`), on the
//! `UpdateProjectiles` stream (`Main.cs:17775-17777`). Every other path
//! (the outbound flight, the recall, the kills, the latches) draws zero.
//! The `Collision.HitTiles` dust + `PlaySound(0)` of the tile arm are the
//! documented dust skip / the `wants_sound` cue.

use crate::projectile::{safe_normalize, Projectile};
use glam::Vec2;
use tr_core::loot::DropRng;

/// `ProjectileID.Flairon` (`ProjectileID.cs:1362`) — the recall head. The
/// port's `FLAIRON` const (data.rs:613) is 1058 FlaironFlail, a DIFFERENT
/// kind (the aiStyle-15 body); this module owns 404 only.
pub const FLAIRON_HEAD: u16 = 404;

/// The `SetDefaults` row (`Projectile.cs:4847-4856`):
/// (width, height, penetrate, alpha); friendly + melee + the stock
/// `timeLeft = 3600` / `tileCollide = true` ride the wiring row.
pub const FLAIRON_HEAD_DEFAULTS: (f32, f32, i32, i32) = (26.0, 26.0, -1, 255);

/// What the arm reads off `Main.player[owner]` each tick (the d5
/// `CrystalLeafOwner` snapshot precedent): the centre and the dead flag.
#[derive(Clone, Copy, Debug)]
pub struct FlaironOwner {
    pub center: Vec2,
    pub dead: bool,
}

/// The arm's player writes (`:30249-30262`), riding back to the wiring
/// caller (the `FlailTick.facing`/`item_rotation` + the golf
/// `dummy_item_time` precedents): `ChangeDir(dir)`,
/// `itemRotation = (−v * dir).ToRotation()`, `SetDummyItemTime(10)`
/// (`Player.cs:4432-4441`).
#[derive(Clone, Copy, Debug)]
pub struct FlaironPlayerWrites {
    pub facing: i32,
    pub item_rotation: f32,
    pub dummy_item_time: i32,
}

/// One queued 405 bubble spawn (`Projectile.NewProjectile` `:30301`): the
/// C# spawn arguments; `ai0 = −10f` is the constant below, the owner rides
/// the slot's. The wiring spawn subtracts the row's half-box
/// (`:10876-10877`, the `FlailSpawnRequest` precedent).
#[derive(Clone, Copy, Debug)]
pub struct FlaironBubbleSpawn {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub knock_back: f32,
}

/// The bubble's `NewProjectile` `ai0` argument (`:30301`'s trailing −10f).
pub const FLAIRON_BUBBLE_AI0: f32 = -10.0;

/// The tick's outcomes: the player writes (None only on the dead-owner
/// early return) and the optional bubble spawn.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlaironTick {
    pub writes: Option<FlaironPlayerWrites>,
    pub spawn: Option<FlaironBubbleSpawn>,
}

/// The slot's `Kill()` shape (the pool's `kill` convention).
fn kill(p: &mut Projectile) {
    p.kill_time_left = p.time_left;
    p.active = false;
    p.time_left = 0;
}

/// `AI()`'s aiStyle-69 arm (`Projectile.cs:30240-30303`), verbatim.
pub fn flairon_step(
    p: &mut Projectile,
    owner: &FlaironOwner,
    local_owner: u8,
    rng: &mut dyn DropRng,
) -> FlaironTick {
    let mut tick = FlaironTick::default();
    let center = Vec2::new(p.pos.x + p.w * 0.5, p.pos.y + p.h * 0.5);
    let v = owner.center - center;
    // `:30243` — the owner-facing rotation (the f32 literal 1.57).
    p.rot = v.y.atan2(v.x) - 1.57;
    // `:30244-30248` — the dead-owner kill.
    if owner.dead {
        kill(p);
        return tick;
    }
    // `:30249-30263` — the player writes + the facing pair.
    let dir = if v.x < 0.0 { 1 } else { -1 };
    p.direction = dir;
    let item_rotation = (v * -1.0 * dir as f32).y.atan2((v * -1.0 * dir as f32).x);
    tick.writes = Some(FlaironPlayerWrites {
        facing: dir,
        item_rotation,
        dummy_item_time: 10,
    });
    // `:30263` — the X == 0 quirk: spriteDirection reads `!(X > 0)`.
    p.sprite_direction = if !(v.x > 0.0) { 1 } else { -1 };
    // `:30264-30267` — the 400-px arm latch.
    if p.ai[0] == 0.0 && v.length() > 400.0 {
        p.ai[0] = 1.0;
    }
    // `:30268-30292` — the recall ladder.
    if p.ai[0] == 1.0 || p.ai[0] == 2.0 {
        let num545 = v.length();
        if num545 > 1500.0 {
            kill(p);
            return tick;
        }
        if num545 > 600.0 {
            p.ai[0] = 2.0;
        }
        p.tile_collide = false;
        let mut num546 = 20.0f32;
        if p.ai[0] == 2.0 {
            num546 = 40.0;
        }
        p.vel = v.normalize() * num546;
        if v.length() < num546 {
            kill(p);
            return tick;
        }
    }
    // `:30293-30297` — the age counter + the alpha fade-in.
    p.ai[1] += 1.0;
    if p.ai[1] > 5.0 {
        p.alpha = 0;
    }
    // `:30298-30302` — the local-owner every-4th-tick bubble spawn.
    if (p.ai[1] as i32) % 4 == 0 && p.owner == local_owner {
        // The speed draw FIRST, then the ONE NextDouble jitter (the f64
        // seam — `Utils.RotatedBy` takes double radians).
        let speed = rng.next_range(45, 65) as f32 * 0.1;
        let jitter = (rng.next_double() - 0.5) * 1.5707963705062866;
        let v49 = crate::bow_launch::rotated_by(safe_normalize(v * -1.0) * speed, jitter);
        tick.spawn = Some(FlaironBubbleSpawn {
            x: center.x,
            y: center.y,
            vx: v49.x,
            vy: v49.y,
            damage: p.damage,
            knock_back: p.knock_back,
        });
    }
    tick
}

/// `Damage_PVE_Inner`'s aiStyle-69 arm (`Projectile.cs:14166-14170`) and
/// `Damage_PVP`'s identical arm (`:14709-14713`): the FIRST hit latches
/// the recall (`ai[0] = 1`) + netUpdate. The wiring caller invokes it on
/// a landed hit (the 404 row's penetrate −1 keeps it alive).
pub fn flairon_hit_latch(p: &mut Projectile) {
    p.ai[0] = 1.0;
    p.net_update = true;
}

/// `HandleMovement`'s blocked-move group (`Projectile.cs:19512-19571`),
/// the 69 half: 404 matches none of the 106/23/383/301 type arms (the
/// `Collision.HitTiles` else `:19548` is the documented dust skip) and
/// none of the 33/106/866 trio, so the generic else latches `ai[0] = 1`
/// (`:19563`) — the velocity flip is the 3/109-only arm (`:19564-19568`),
/// NOT 69's — then netUpdate (`:19570`) + `PlaySound(0, (int)x, (int)y)`
/// (`:19571`, the int overload: style 0). The head does NOT die on the
/// tile; it recalls.
pub fn flairon_tile_hit(p: &mut Projectile) {
    p.ai[0] = 1.0;
    p.net_update = true;
    p.wants_sound = Some((0, 0, p.pos.x, p.pos.y, 0.0));
}

/// `HandleMovement`'s type-405 blocked-move arm (`Projectile.cs:18317-18335`,
/// W4-d7 F5): `localAI[0]++`; the FIRST bounce (`localAI[0] < 2`) reflects
/// each changed axis from `lastVelocity` (full speed, no damping); the
/// SECOND (`localAI[0] >= 2`) `Kill()`s. The wiring caller invokes it when
/// the move was blocked (the `:18398` gate is NOT reached — the 405 type arm
/// precedes it), with the pre-collision velocity. Returns true when the
/// bubble died (the second bounce).
pub fn bubble_bounce(p: &mut Projectile, last_vel: Vec2) -> bool {
    p.local_ai[0] += 1.0;
    if p.local_ai[0] < 2.0 {
        if p.vel.x != last_vel.x {
            p.vel.x = 0.0 - last_vel.x;
        }
        if p.vel.y != last_vel.y {
            p.vel.y = 0.0 - last_vel.y;
        }
        false
    } else {
        // `:18333` — `Kill()` (the pool's kill convention).
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
        true
    }
}

/// `Kill()`'s type-405 arm (`Projectile.cs:72849-72873`, W4-d7 F5): sound
/// Item54 + 20× dust 212, each ONE `NextDouble` hue draw (`:72868`). The
/// `num286 < 30` branch (`:72860`) is ALWAYS taken (the loop runs 0..20), so
/// the `Next(45,91)` else arm (`:72866`) is DEAD — zero draws there. The 20
/// hue draws stay on the stream (the bodies skipped); the sound cue rides
/// the pool's death hook.
pub fn bubble_kill_draws(rng: &mut dyn DropRng) {
    for _ in 0..20 {
        let _ = rng.next_double(); // `:72868` — the hslToRgb hue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::UnifiedRandom;

    fn head(pos: Vec2, vel: Vec2) -> Projectile {
        Projectile {
            active: true,
            kind: FLAIRON_HEAD,
            owner: 0,
            pos,
            vel,
            w: FLAIRON_HEAD_DEFAULTS.0,
            h: FLAIRON_HEAD_DEFAULTS.1,
            penetrate: FLAIRON_HEAD_DEFAULTS.2,
            max_penetrate: FLAIRON_HEAD_DEFAULTS.2,
            alpha: FLAIRON_HEAD_DEFAULTS.3,
            time_left: 3600,
            tile_collide: true,
            friendly: true,
            melee: true,
            damage: 66,
            knock_back: 4.5,
            ..Default::default()
        }
    }

    fn owner_at(cx: f32, cy: f32) -> FlaironOwner {
        FlaironOwner { center: Vec2::new(cx, cy), dead: false }
    }

    /// `:30242-30248` — the rotation faces the owner BEFORE the dead
    /// check; a dead owner kills and returns no writes, no draws.
    #[test]
    fn w4_d7_s69_the_dead_owner_kills_after_the_rotation() {
        let mut rng = UnifiedRandom::new(69);
        let mut control = UnifiedRandom::new(69);
        // Head centre (113, 113); owner (113, 613): v = (0, 500).
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::new(14.0, 0.0));
        let mut o = owner_at(113.0, 613.0);
        o.dead = true;
        let tick = flairon_step(&mut p, &o, 0, &mut rng);
        assert!(!p.active);
        assert!((p.rot - (std::f32::consts::FRAC_PI_2 - 1.57)).abs() < 1e-5);
        assert!(tick.writes.is_none() && tick.spawn.is_none());
        assert_eq!(rng.next_range(0, 1), control.next_range(0, 1), "zero draws");
    }

    /// `:30249-30263` — the writes + the facing pair, incl. the X == 0
    /// quirk (direction −1, spriteDirection +1).
    #[test]
    fn w4_d7_s69_the_player_writes_and_the_zero_x_quirk() {
        let mut rng = UnifiedRandom::new(1);
        // Owner directly BELOW: v = (0, 500), X == 0.
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        let tick = flairon_step(&mut p, &owner_at(113.0, 613.0), 0, &mut rng);
        let w = tick.writes.expect("the live owner writes");
        assert_eq!(w.facing, -1);
        assert_eq!(w.dummy_item_time, 10);
        // itemRotation = (−v * −1).ToRotation() = (0, 500) → +π/2.
        assert!((w.item_rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert_eq!(p.direction, -1);
        assert_eq!(p.sprite_direction, 1, "!(0 > 0) → +1");
        // Owner to the LEFT: v.X < 0 → direction +1, spriteDirection +1.
        let mut q = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        let t2 = flairon_step(&mut q, &owner_at(13.0, 113.0), 0, &mut rng);
        assert_eq!(t2.writes.unwrap().facing, 1);
        assert_eq!(q.direction, 1);
        assert_eq!(q.sprite_direction, 1);
        // Owner to the RIGHT: v.X > 0 → direction −1, spriteDirection −1.
        let mut r = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        let t3 = flairon_step(&mut r, &owner_at(213.0, 113.0), 0, &mut rng);
        assert_eq!(t3.writes.unwrap().facing, -1);
        assert_eq!(r.sprite_direction, -1);
    }

    /// `:30264-30292` — the 400-px latch arms the recall the SAME tick:
    /// state 1, speed 20 toward the owner, tileCollide cleared.
    #[test]
    fn w4_d7_s69_the_400px_latch_arms_the_recall() {
        let mut rng = UnifiedRandom::new(2);
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::new(14.0, 0.0));
        // v = (0, 500): > 400 latches, <= 600 keeps state 1.
        flairon_step(&mut p, &owner_at(113.0, 613.0), 0, &mut rng);
        assert_eq!(p.ai[0], 1.0);
        assert!(!p.tile_collide);
        assert!((p.vel.x).abs() < 1e-5 && (p.vel.y - 20.0).abs() < 1e-5);
        // A near owner (<= 400) keeps the outbound state 0.
        let mut q = head(Vec2::new(100.0, 100.0), Vec2::new(14.0, 0.0));
        flairon_step(&mut q, &owner_at(113.0, 413.0), 0, &mut rng);
        assert_eq!(q.ai[0], 0.0);
        assert!(q.tile_collide);
        assert_eq!(q.vel, Vec2::new(14.0, 0.0), "the outbound velocity holds");
    }

    /// `:30271-30291` — the ladder: > 1500 kills; > 600 escalates to
    /// state 2 / speed 40; the arrival (|v| < speed) kills.
    #[test]
    fn w4_d7_s69_the_recall_ladder() {
        let mut rng = UnifiedRandom::new(3);
        // 1600 px away, already armed → the cut kill (writes still ran).
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        p.ai[0] = 1.0;
        let tick = flairon_step(&mut p, &owner_at(113.0, 1713.0), 0, &mut rng);
        assert!(!p.active);
        assert!(tick.writes.is_some(), "the writes precede the ladder");
        // 700 px away → state 2, speed 40.
        let mut q = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        q.ai[0] = 1.0;
        flairon_step(&mut q, &owner_at(113.0, 813.0), 0, &mut rng);
        assert_eq!(q.ai[0], 2.0);
        assert!((q.vel.y - 40.0).abs() < 1e-4);
        // 10 px away, armed → the arrival kill (10 < 20).
        let mut r = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        r.ai[0] = 1.0;
        flairon_step(&mut r, &owner_at(113.0, 123.0), 0, &mut rng);
        assert!(!r.active);
    }

    /// `:30293-30297` — the age counter ticks every path that survives;
    /// alpha pins 0 from the 6th tick.
    #[test]
    fn w4_d7_s69_the_age_counter_and_alpha() {
        let mut rng = UnifiedRandom::new(4);
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        // A near owner keeps state 0 (no recall, no spawn before tick 4).
        for i in 1..=6 {
            flairon_step(&mut p, &owner_at(113.0, 213.0), 0, &mut rng);
            assert_eq!(p.ai[1], i as f32);
            if i <= 5 {
                assert_eq!(p.alpha, 255);
            } else {
                assert_eq!(p.alpha, 0);
            }
        }
    }

    /// `:30298-30302` — tick 4 spawns ONE bubble: Next(45,65) speed then
    /// ONE NextDouble jitter, the −v direction, the slot's damage/KB, the
    /// centre spawn point; the oracle replay pins the draw order.
    #[test]
    fn w4_d7_s69_the_bubble_spawn_draw_order() {
        let mut rng = UnifiedRandom::new(69);
        let mut oracle = UnifiedRandom::new(69);
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        p.ai[1] = 3.0; // the next tick is the 4th
        // v = (0, 100): state 0 (<= 400), so only the spawn path draws.
        let tick = flairon_step(&mut p, &owner_at(113.0, 213.0), 0, &mut rng);
        let s = tick.spawn.expect("the 4th tick spawns");
        let speed = oracle.next_range(45, 65) as f32 * 0.1;
        let jitter = (oracle.next_double() - 0.5) * 1.5707963705062866;
        let want = crate::bow_launch::rotated_by(Vec2::new(0.0, -1.0) * speed, jitter);
        assert!((s.vx - want.x).abs() < 1e-5 && (s.vy - want.y).abs() < 1e-5);
        assert_eq!((s.x, s.y), (113.0, 113.0));
        assert_eq!(s.damage, 66);
        assert!((s.knock_back - 4.5).abs() < 1e-6);
        assert_eq!(rng.next_range(0, 1), oracle.next_range(0, 1), "exactly two draws");
        // Ticks 1-3 and 5-7 spawn nothing.
        let mut q = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        for i in 1..=7 {
            let t = flairon_step(&mut q, &owner_at(113.0, 213.0), 0, &mut rng);
            assert_eq!(t.spawn.is_some(), i == 4, "tick {i}");
        }
    }

    /// `:30298` — a foreign-owned head never spawns and never draws.
    #[test]
    fn w4_d7_s69_the_spawn_is_local_owner_only() {
        let mut rng = UnifiedRandom::new(5);
        let mut control = UnifiedRandom::new(5);
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::ZERO);
        p.owner = 1;
        p.ai[1] = 3.0;
        let tick = flairon_step(&mut p, &owner_at(113.0, 213.0), 0, &mut rng);
        assert!(tick.spawn.is_none());
        assert_eq!(rng.next_range(0, 1), control.next_range(0, 1), "zero draws");
    }

    /// `:14166-14170` / `:14709-14713` — the hit latch arms the recall.
    #[test]
    fn w4_d7_s69_the_hit_latch() {
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::new(14.0, 0.0));
        flairon_hit_latch(&mut p);
        assert_eq!(p.ai[0], 1.0);
        assert!(p.net_update);
        assert!(p.active, "penetrate −1 — the hit does not kill");
    }

    /// `:19512-19571` — the tile block latches the recall with NO velocity
    /// flip (the 3/109-only arm) and queues the int-overload sound 0.
    #[test]
    fn w4_d7_s69_the_tile_hit_latches_without_a_flip() {
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::new(14.0, 3.0));
        flairon_tile_hit(&mut p);
        assert_eq!(p.ai[0], 1.0);
        assert!(p.net_update);
        assert_eq!(p.vel, Vec2::new(14.0, 3.0), "69 keeps its velocity");
        assert_eq!(p.wants_sound, Some((0, 0, 100.0, 100.0, 0.0)));
        assert!(p.active, "the head recalls, it does not die on the tile");
    }

    /// W4-d7 F5 — `:18317-18335`: the first bounce reflects each changed axis
    /// at full speed; the second kills. An unchanged axis keeps its value.
    #[test]
    fn w4_d7_f5_the_bubble_bounce_ladder() {
        let mut p = head(Vec2::new(100.0, 100.0), Vec2::new(0.0, 4.0));
        p.kind = 405;
        // First block: X changed (6→0), Y unchanged (4→4).
        let died = bubble_bounce(&mut p, Vec2::new(6.0, 4.0));
        assert!(!died);
        assert_eq!(p.local_ai[0], 1.0);
        assert!((p.vel.x - (-6.0)).abs() < 1e-6, "X reflects full speed");
        assert_eq!(p.vel.y, 4.0, "Y unchanged — no reflect");
        assert!(p.active);
        // Second block: both axes changed → Kill.
        p.vel = Vec2::new(0.0, 0.0);
        let died = bubble_bounce(&mut p, Vec2::new(-6.0, 4.0));
        assert!(died);
        assert_eq!(p.local_ai[0], 2.0);
        assert!(!p.active, "the second bounce kills");
        assert_eq!(p.time_left, 0);
    }

    /// W4-d7 F5 — `:72849-72873`: the Kill census draws exactly 20 NextDouble
    /// hue values (the num286<30 branch always taken, the Next(45,91) dead).
    #[test]
    fn w4_d7_f5_the_bubble_kill_draws_twenty_hues() {
        let mut rng = UnifiedRandom::new(405);
        let mut oracle = UnifiedRandom::new(405);
        bubble_kill_draws(&mut rng);
        for _ in 0..20 {
            let _ = oracle.next_double();
        }
        assert_eq!(rng.next_range(0, 1), oracle.next_range(0, 1), "exactly 20 draws");
    }
}
