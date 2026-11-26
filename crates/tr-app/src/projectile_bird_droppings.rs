//! W4-c10 — the BirdDroppings 1080's projectile AI: `AI()`'s
//! `else if (this.aiStyle == 200)` dispatch (`Projectile.cs:36763-36766`)
//! into `AI_200_BirdDroppings` (`Projectile.cs:39985-40017`), the shot the
//! aiStyle-24 birds (`NPC.cs:25322`) and the aiStyle-68 waterfowl
//! (`NPC.cs:34632`) drop on the player below them.
//!
//! The arm, verbatim:
//! * `velocity.Y += 0.3f` with the 16f cap (`:39987-39991`) — the drop's
//!   own gravity (NOT the aiStyle-1 flight's 15-tick/0.1 ladder);
//! * the TWO-dust trail (`:39992-40008`): `Dust.NewDust(Center, 1, 1,
//!   329)` per iteration, `velocity *= 0.1f`, then the `Main.rand.Next(2)`
//!   gate — hit: `scale = NextFloat() * 0.5f + 0.5f`, `velocity +=
//!   base.velocity`, `noGravity = true`; miss: `scale = NextFloat() *
//!   0.4f + 0.4f`, `velocity *= 0.5f`, `velocity += base.velocity / 2f`.
//!   The dust VISUALS are the port's documented dust-layer skip (the
//!   frost-shot precedent, `projectile_frost_shots.rs`), but the DRAWS
//!   ride the pool's `UpdateProjectiles` stream in the C#'s exact order —
//!   the 480 flame drop's census convention (`projectile_ai_darts.rs::
//!   flame_drop_dust`): per iteration ONE `Next(2)` then ONE `NextFloat`,
//!   two iterations per tick;
//! * the player-intersect kill (`:40009-40016`): the int-truncated
//!   10x10 rect vs every active-and-alive player's int-truncated rect,
//!   XNA `Rectangle.Intersects` (strict edges) → `Kill()`. The port has
//!   one session player; the seam carries its rect (a `None` player —
//!   dead/absent — never kills). NOTE the kill is DAMAGE-free: the C#
//!   `Kill()` only deactivates (the drop's `damage` is 0 from the spawn
//!   call, `NPC.cs:34632` — `Damage_PVP` refuses it, `:14775`).
//!
//! The DEATH hook (`Kill()`'s type-1080 arm, `Projectile.cs:70926-70954`,
//! W4-c10 sweep): EVERY death path that goes through `Kill()` — the
//! player-intersect kill above (`:40014`), the `HandleMovement` splat
//! (`:18926`), the lifetime/penetrate expiries (`:17204`/`:17208`) — runs
//! it: the Stinky buff 120 for 600 ticks on the local player while alive
//! whose int-truncated rect intersects the drop's (`ApplyBuffTo`,
//! `:12391-12398` — the `trap && deadMansSweater` halve never fires: the
//! row's `trap` is false), the `SoundID.Item177` cue at the int-truncated
//! CENTRE (`:70936` — the call passes `(int)base.Center.X/Y`), and the
//! 30-dust burst's DRAW census (`:70937-70951`:
//! per iteration ONE `NextFloat` (the Y lift, drawn UNCONDITIONALLY) then
//! ONE `Next(2)` gate then ONE `NextFloat` (the scale) — three draws ×
//! 30). The dust bodies are the documented visual skip; the cue rides
//! `wants_sound` and the pool forwards it at the death hook (the cue
//! drain sits ahead of the AI, so a hook-set intent needs the forward at
//! the hook — the `moonlord_arrow_kill` once-per-death seam). The
//! out-of-world guard's direct deactivate (`:16696`) never calls `Kill()`
//! and skips the hook — the pool's `died_out_of_world` gate carries that.
//!
//! Movement is NOT in the arm — `HandleMovement`'s type-1080 branch
//! (`Projectile.cs:18921-18928`): after the tile collision, `if
//! (velocity != lastVelocity) { position += velocity; Kill(); }` — the
//! drop splats on the first blocked step. The port's stand-in is the
//! `kill_step` shape (`projectile.rs`, the frost-shot `move_tile_collide`
//! precedent): the blocked-axis probe, the integrate-then-kill on a hit.
//! `ignoreWater = true` (`Projectile.cs:10085`) keeps the wet slowdown
//! out (`NewProjectile`'s `wet = false` write, `:10886-10889`).
//!
//! CODE-ONLY MODE (W4-c10): the witness tests at the bottom are WRITTEN
//! NOT RUN (no compiler invoked); every expected value is hand-derived
//! from the cited C# lines.

use tr_core::loot::DropRng;
use tr_core::World;

use crate::projectile::{Projectile, BIRD_DROPPINGS};

/// The dispatch gate: only the 1080 rides this module (the pool's
/// `match data.ai_style` 200 arm — the style's only user).
pub(crate) fn handles(kind: u16) -> bool {
    kind == BIRD_DROPPINGS
}

/// One tick of the drop: `AI_200_BirdDroppings` (`Projectile.cs:
/// 39985-40017`) + the `HandleMovement` type-1080 tail (`:18921-18928`).
/// `player_rect` is the session player's int-truncated
/// `(x, y, width, height)` while active-and-alive (`:40012`'s gate);
/// `rng` is the pool's `UpdateProjectiles` stream (the dust census).
pub(crate) fn bird_droppings_step(
    p: &mut Projectile,
    world: &World,
    player_rect: Option<(i32, i32, i32, i32)>,
    rng: &mut dyn DropRng,
) {
    // `:39987-39991` — the 0.3 gravity, the 16f cap.
    p.vel.y += 0.3;
    if p.vel.y > 16.0 {
        p.vel.y = 16.0;
    }
    // `:39992-40008` — the two-dust trail's draw census: per iteration
    // the `Next(2)` gate THEN the branch's one `NextFloat` (both branches
    // draw exactly one float — the scale roll), in order. The dust
    // spawns/velocity writes are the documented visual skip.
    for _ in 0..2 {
        let _ = rng.next_range(0, 2);
        let _ = rng.next_float();
    }
    // `:40009-40016` — the player-intersect kill (the int-truncated
    // rects, XNA Rectangle.Intersects' strict edges). `Kill()`'s
    // observable subset: timeLeft 0 + active false (`:70650`, the
    // port's `kill` shape; the channel-cancel head is a no-op for the
    // NPC-owned drop). The Kill arm's buff/cue/census ride the pool's
    // death hook (`bird_droppings_kill_draws` + the 1080 hook there).
    if let Some((px, py, pw, ph)) = player_rect {
        let (bx, by) = (p.pos.x as i32, p.pos.y as i32);
        let (bw, bh) = (p.w as i32, p.h as i32);
        if bx < px + pw && px < bx + bw && by < py + ph && py < by + bh {
            p.kill_time_left = p.time_left;
            p.active = false;
            p.time_left = 0;
            // The corpse drift (the W4-d20 compare, sweep): the C#'s
            // `HandleMovement` + `UpdatePosition` (`:17015`/`:19858` →
            // `:20445`) run BEFORE the `!active` return (`:17027`), so
            // the killed drop still integrates one velocity step (the
            // air case: `velocity == lastVelocity`, the `:18923` splat
            // arm is a no-op). The blocked-tile double-step granularity
            // stays the documented probe stand-in.
            p.pos += p.vel;
            return;
        }
    }
    // `HandleMovement`'s type-1080 branch (`:18921-18928`): the tile
    // collision zeroes the blocked axes; `velocity != lastVelocity` →
    // `position += velocity; Kill()`. The port's blocked-axis probe
    // stand-in (the frost-shot `move_tile_collide` shape): a blocked
    // step integrates then kills; a free step integrates and lives.
    let nx = p.pos.x + p.vel.x;
    let ny = p.pos.y + p.vel.y;
    let hit_x = crate::player::collides_rect(world, nx, p.pos.y, p.w, p.h);
    let hit_y = crate::player::collides_rect(world, p.pos.x, ny, p.w, p.h);
    p.pos.x = nx;
    p.pos.y = ny;
    if hit_x || hit_y {
        p.kill_time_left = p.time_left;
        p.active = false;
        p.time_left = 0;
    }
}

/// The `Kill()` type-1080 arm's draw census (`Projectile.cs:70937-70951`):
/// THIRTY dust-329 iterations, each drawing — in the C#'s statement order
/// — the Y-lift `NextFloat()` (`:70942`, UNCONDITIONAL, ahead of the
/// gate), then the `Next(2)` branch pick (`:70943`), then the branch's
/// one `NextFloat()` scale (`:70946` hit / `:70950` miss — both branches
/// draw exactly one). 90 draws per death, on the pool's
/// `UpdateProjectiles` stream. The dust bodies (`velocity *= 0.8`, the
/// `Y *= 0.2` squash, the lift, noGravity, the scales) are the documented
/// visual skip. The buff + the Item177 cue are the pool hook's (they need
/// the player seam / the sound queue).
pub(crate) fn bird_droppings_kill_draws(rng: &mut dyn DropRng) {
    for _ in 0..30 {
        let _ = rng.next_float(); // velocity.Y -= NextFloat() * 3f
        let _ = rng.next_range(0, 2); // the noGravity gate
        let _ = rng.next_float(); // scale = NextFloat() * 0.6 + 0.6 / * 0.5 + 0.5
    }
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
    use tr_core::tile::{Tile, FLAG_ACTIVE};

    /// A recording stream: every draw lands in order (the darts module's
    /// Recorder shape).
    struct Recorder {
        draws: Vec<&'static str>,
        ranges: Vec<(i32, i32)>,
        next: i32,
    }
    impl Recorder {
        fn new(next: i32) -> Self {
            Self { draws: Vec::new(), ranges: Vec::new(), next }
        }
    }
    impl DropRng for Recorder {
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.draws.push("range");
            self.ranges.push((min, max));
            self.next
        }
        fn next_float(&mut self) -> f32 {
            self.draws.push("float");
            0.5
        }
    }

    fn world() -> World {
        World::new("t".into(), 200, 120)
    }

    fn drop_shot(x: f32, y: f32, vy: f32) -> Projectile {
        let d = proj_data(BIRD_DROPPINGS).expect("the 1080 row");
        let mut p = Projectile::default();
        p.active = true;
        p.kind = BIRD_DROPPINGS;
        p.pos = Vec2::new(x, y);
        p.vel = Vec2::new(0.0, vy);
        p.w = d.w;
        p.h = d.h;
        p.time_left = d.time_left;
        p.penetrate = d.penetrate;
        p.alpha = d.alpha;
        p.tile_collide = d.tile_collide;
        p
    }

    /// `Projectile.cs:10078-10086` — the SetDefaults row: 10x10, aiStyle
    /// 200, alpha 255, penetrate -1, ignoreWater, the stock tileCollide
    /// true + timeLeft 3600.
    #[test]
    fn the_1080_row_matches_set_defaults() {
        let d = proj_data(BIRD_DROPPINGS).expect("the row");
        assert_eq!((d.w, d.h), (10.0, 10.0));
        assert_eq!(d.ai_style, 200);
        assert_eq!(d.alpha, 255);
        assert_eq!(d.penetrate, -1);
        assert!(d.ignore_water);
        assert!(d.tile_collide, "the stock :775 stands");
        assert_eq!(d.time_left, 3600, "the stock :787 stands");
        assert!(handles(BIRD_DROPPINGS));
        assert!(!handles(1081));
    }

    /// `Projectile.cs:39987-39991` + `:39992-40008` — the 0.3 gravity /
    /// 16 cap and the draw census: exactly TWO (Next(2), NextFloat) pairs
    /// per tick, in order, whether or not the gate hits.
    #[test]
    fn the_drop_falls_and_censuses_two_dust_rolls() {
        let w = world();
        for gate in [0, 1] {
            let mut p = drop_shot(1000.0, 1000.0, 1.0);
            let mut rng = Recorder::new(gate);
            bird_droppings_step(&mut p, &w, None, &mut rng);
            assert_eq!(p.vel.y, 1.3, "the 0.3 gravity (:39987)");
            assert_eq!(
                rng.draws,
                vec!["range", "float", "range", "float"],
                "gate {gate}: one Next(2) + one NextFloat per dust"
            );
            assert_eq!(rng.ranges, vec![(0, 2), (0, 2)]);
            assert!(p.active);
            assert_eq!(p.pos, Vec2::new(1000.0, 1001.3), "the free integrate");
        }
        // The 16f cap.
        let mut p = drop_shot(1000.0, 1000.0, 15.9);
        bird_droppings_step(&mut p, &w, None, &mut Recorder::new(0));
        assert_eq!(p.vel.y, 16.0, "the cap (:39988-39991)");
    }

    /// `Projectile.cs:40009-40016` — the player-intersect kill: the
    /// int-truncated rects, the strict edges; the kill is damage-free
    /// (Kill() only deactivates).
    #[test]
    fn the_drop_kills_on_the_player_rect() {
        let w = world();
        // The drop at (1000, 1000) 10x10 vs the player (1005, 1005)
        // 20x42: overlap → kill, then the corpse drift (the C#'s
        // UpdatePosition runs ahead of the `!active` return, `:17027`).
        let mut p = drop_shot(1000.0, 1000.0, 1.0);
        bird_droppings_step(&mut p, &w, Some((1005, 1005, 20, 42)), &mut Recorder::new(0));
        assert!(!p.active, "the intersect kill (:40014)");
        assert_eq!(p.time_left, 0);
        assert_eq!(
            p.pos,
            Vec2::new(1000.0, 1001.3),
            "the corpse drift: one gravity-stepped integrate (:20445)"
        );
        // The strict edge: the player's left at 1010 = the drop's right
        // — NO intersect (XNA's half-open compare).
        let mut p2 = drop_shot(1000.0, 1000.0, 1.0);
        bird_droppings_step(&mut p2, &w, Some((1010, 1000, 20, 42)), &mut Recorder::new(0));
        assert!(p2.active, "the strict edge stands");
        // A dead/absent player (None) never kills.
        let mut p3 = drop_shot(1000.0, 1000.0, 1.0);
        bird_droppings_step(&mut p3, &w, None, &mut Recorder::new(0));
        assert!(p3.active);
    }

    /// `Projectile.cs:18921-18928` — the HandleMovement tail: a blocked
    /// step integrates the (collision-zeroed) velocity then kills; the
    /// port's probe stand-in integrates the full step then kills (the
    /// frost-shot `move_tile_collide` granularity, documented).
    #[test]
    fn the_drop_splats_on_a_blocked_step() {
        let mut w = world();
        // A solid floor at row 64 (y 1024): the drop at (1000, 1016)
        // with vy 1.3 steps to 1017.3 — its 10-px box reaches 1027.3,
        // inside the floor → the blocked-Y kill.
        for x in 60..68 {
            let i = w.index(x, 64);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = drop_shot(1000.0, 1016.0, 1.0);
        bird_droppings_step(&mut p, &w, None, &mut Recorder::new(0));
        assert!(!p.active, "the blocked step kills (:18926)");
        assert_eq!(p.time_left, 0);
        // The free air above keeps flying.
        let mut p2 = drop_shot(1000.0, 900.0, 1.0);
        bird_droppings_step(&mut p2, &w, None, &mut Recorder::new(0));
        assert!(p2.active);
        assert_eq!(p2.pos.y, 901.3);
    }

    /// `Projectile.cs:70937-70951` — the Kill arm's 30-dust census: per
    /// iteration the UNCONDITIONAL Y-lift NextFloat FIRST, then the
    /// Next(2) gate, then the branch's one NextFloat scale — 90 draws in
    /// that exact order, both gate outcomes.
    #[test]
    fn the_kill_burst_censuses_thirty_dust_rows() {
        for gate in [0, 1] {
            let mut rng = Recorder::new(gate);
            bird_droppings_kill_draws(&mut rng);
            assert_eq!(rng.draws.len(), 90, "30 × (float, range, float)");
            assert_eq!(rng.ranges.len(), 30);
            assert!(rng.ranges.iter().all(|&r| r == (0, 2)));
            for i in 0..30 {
                assert_eq!(rng.draws[i * 3], "float", "the Y lift draws first (:70942)");
                assert_eq!(rng.draws[i * 3 + 1], "range", "the gate (:70943)");
                assert_eq!(rng.draws[i * 3 + 2], "float", "the scale (:70946/:70950)");
            }
        }
    }

    /// `Projectile.cs:70926-70936` + `:12391-12398` — the pool's death
    /// hook: a drop dying ON the living player grants Stinky 120 for 600
    /// ticks and queues Item177 at the int centre; a death away from the
    /// player grants nothing but still plays + censuses.
    #[test]
    fn the_kill_hook_grants_stinky_and_queues_item177() {
        use crate::projectile::{FlailInputs, ProjectilePool, YoyoInputs};
        let mut w = world();
        let mut pl = crate::player::Player::spawn(&w);
        pl.x = 1000.0;
        pl.y = 1000.0;
        let mut pool = ProjectilePool::new();
        // time_left 1: the lifetime expiry (`:17204`) is the Kill() path.
        let s = pool.spawn_hostile(BIRD_DROPPINGS, Vec2::new(1005.0, 1005.0), Vec2::ZERO, 0, 0.0).unwrap();
        pool.set_time_left(s, 1);
        let mut npcs = Vec::new();
        pool.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut Recorder::new(0),
            0,
            true,
        );
        assert!(!pool.slots()[s].active, "the expiry killed the drop");
        let stink = pl.buffs.buff_type.iter().position(|&b| b == 120);
        assert!(stink.is_some(), "Stinky 120 granted (:70933)");
        assert_eq!(pl.buffs.buff_time[stink.unwrap()], 600);
        let cues = pool.drain_proj_sounds();
        assert!(
            cues.contains(&(2, 177, 1010.0, 1010.0, 0.0, 1.0)),
            "Item177 at the int centre (:70936); got {cues:?}"
        );
        // A death AWAY from the player: no buff, but the cue stands.
        let mut pl2 = crate::player::Player::spawn(&w);
        pl2.x = 1000.0;
        pl2.y = 1000.0;
        let mut pool2 = ProjectilePool::new();
        let s2 = pool2.spawn_hostile(BIRD_DROPPINGS, Vec2::new(1500.0, 900.0), Vec2::ZERO, 0, 0.0).unwrap();
        pool2.set_time_left(s2, 1);
        pool2.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl2),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut Recorder::new(0),
            0,
            true,
        );
        assert!(!pl2.buffs.buff_type.contains(&120), "no intersect, no buff");
        assert!(pool2.drain_proj_sounds().contains(&(2, 177, 1505.0, 905.0, 0.0, 1.0)));
        // A DEAD player never gains the buff (`:70928`'s gate).
        let mut pl3 = crate::player::Player::spawn(&w);
        pl3.x = 1000.0;
        pl3.y = 1000.0;
        pl3.dead = true;
        let mut pool3 = ProjectilePool::new();
        let s3 = pool3.spawn_hostile(BIRD_DROPPINGS, Vec2::new(1005.0, 1005.0), Vec2::ZERO, 0, 0.0).unwrap();
        pool3.set_time_left(s3, 1);
        pool3.update(
            &mut w,
            &mut npcs,
            None,
            None,
            Some(&mut pl3),
            YoyoInputs::default(),
            FlailInputs::default(),
            &mut Recorder::new(0),
            0,
            true,
        );
        assert!(!pl3.buffs.buff_type.contains(&120), "the dead player is skipped");
    }
}
